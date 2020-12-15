extern crate tokio;
extern crate unshare;

use std;
use std::error::Error;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::net::UnixListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio::stream::StreamExt;
use tokio::sync::mpsc;
use tokio::time::delay_for;
use tracing::{debug, error, info};
//use uuid;
use rand::Rng;
use rand_core::SeedableRng;
use tokio::task;

// For rtnetlink
//use tokio::stream::TryStreamExt;
use ipnetwork::{IpNetwork, Ipv4Network};
use rtnetlink::{new_connection, Error as RtNetlinkError, Handle};

use crate::application::config::{Application, ApplicationConfig};
use crate::common::*;
use crate::context::{Context, ContextReference, ServiceIndex};
use crate::device;
use crate::initrc::UnitType;
use crate::mount;
use crate::network;
use crate::server;
use crate::sysfs_walker;
use crate::ueventd;
use crate::zygote;

pub fn initception_main(pid1: bool) -> Result<(), Box<dyn Error>> {
    if pid1 {
        if let Err(e) = device::mount_basics() {
            error!("Unable to mount basics");
            return Err(e);
        }
    }

    if pid1 {
        if let Err(e) = device::make_basic_devices() {
            error!("Unable to make basic devices");
            return Err(e);
        }
    }

    if let Err(e) = zygote::launch_zygote() {
        error!("Error launching zygote");
        return Err(e);
    }

    let context = Context::create_context().unwrap();
    let context = Arc::new(RwLock::new(context));
    //info!("Loaded config : {:?}", context);
    match init_async_main(context) {
        Err(e) => Err(Box::new(e)),
        Ok(()) => {
            error!("init terminating");
            Ok(())
        }
    }
}

pub fn initception_main_static(
    configs: &[&dyn ApplicationConfig],
    is_pid1: bool,
) -> Result<(), Box<dyn Error>> {
    if is_pid1 {
        if let Err(e) = device::mount_basics() {
            error!("Unable to mount basics");
            return Err(e);
        }
    }

    if is_pid1 {
        if let Err(e) = device::make_basic_devices() {
            error!("Unable to make basic devices");
            return Err(e);
        }
    }

    let mut context = Context::new();
    for app in configs {
        context.add_service(*app);
    }

    /*
    Todo: Add units here
    context.add_unit(unit);
    */

    context.fixup_dependencies();
    let context = Arc::new(RwLock::new(context));

    match init_async_main(context) {
        Err(e) => Err(Box::new(e)),
        Ok(()) => {
            error!("init terminating");
            Ok(())
        }
    }
}

// We don't need a crypto backed uuid here
fn create_uuid(rng: &mut rand::rngs::SmallRng) -> String {
    let mut uuid = String::new();
    for _i in 0..32 {
        let num: u8 = rng.gen::<u8>() % 32 as u8 + 48u8;
        uuid.push(num as char);
    }
    println!("uuid:{}", &uuid);
    uuid
}

#[tokio::main]
async fn init_async_main(context: ContextReference) -> Result<(), std::io::Error> {
    let (tx_orig, rx) = std::sync::mpsc::channel::<TaskMessage>();

    {
        let initial_services = context.read().unwrap().get_initial_services();
        let tx = tx_orig.clone();
        info!("asyn main started");
        if let Err(_) = tx.send(TaskMessage::ConfigureNetworkLoopback) {
            panic!("Receiver dropped when configuring network");
        }

        for service_idx in initial_services {
            if let Err(_) = tx.send(TaskMessage::RequestLaunch(service_idx)) {
                panic!("Receiver dropped");
            }
        }
    }

    // Spawn the task for uevent processing
    let tx = tx_orig.clone();
    // uevent calls blocking functions.
    tokio::task::spawn_blocking(|| ueventd::uevent_main(tx));

    let cloned_context = context.clone();
    // Needed for the signal function below
    let mut tx = tx_orig.clone();

    let rng_main = Arc::new(Mutex::new(rand::rngs::SmallRng::from_seed([
        2, 5, 6, 7, 4, 3, 5, 6, 7, 5, 4, 3, 5, 6, 7, 7,
    ])));

    // This is the main dispatch function for initception
    tokio::task::spawn_blocking(move || {
        while let Ok(msg) = rx.recv() {
            let cloned_context = context.clone();
            let mount_context = context.clone();
            let mut tx = tx_orig.clone();
            let rng = rng_main.clone();
            match msg {
                TaskMessage::ConfigureNetworkLoopback => tokio::spawn(async move {
                    //debug!("Configure Loopback network interface");
                    //let ip = IpNetwork::V4("127.0.0.1".parse().unwrap());
                    //network::configure_network_interface(ip, String::from("lo")).await
                    debug!("Loopback network set up (skipped)");
                }),
                TaskMessage::ProcessRunning(id) => tokio::spawn(async move {
                    debug!("Process Running {:?}", id);
                    let deps = cloned_context.read().unwrap().get_immediate_dependants(id);
                    for dep in deps {
                        debug!("Launching dep {:?}", dep);

                        if let Err(_) = tx.send(TaskMessage::RequestLaunch(dep)) {
                            panic!("Receiver dropped");
                        }
                    }
                }),
                TaskMessage::ProcessLaunched(id) => tokio::spawn(async move {
                    debug!("Process launched {:?}", id);
                    let deps = cloned_context
                        .read()
                        .unwrap()
                        .get_immediate_dependant_services(id);
                    for dep in deps {
                        debug!("Launching dep {:?}", dep);
                        if let Err(_) = tx.send(TaskMessage::RequestLaunch(dep)) {
                            panic!("Receiver dropped");
                        }
                    }
                }),
                TaskMessage::ProcessExited(id) => tokio::spawn(async move {
                    debug!("Pid {:?} has exited", id);
                    if let Some(context) = cloned_context.write().unwrap().get_service(id) {
                        context.write().unwrap().cleanup_resources();
                    }

                    if let Some(time_ms) = cloned_context.read().unwrap().check_restart(id) {
                        tokio::spawn(async move {
                            delay_for(Duration::from_millis(time_ms as u64)).await;
                            if let Err(_) = tx.send(TaskMessage::RequestLaunch(id)) {
                                panic!("Receiver dropped");
                            }
                        });
                    }
                }),
                TaskMessage::ProcessPaused(id) => tokio::spawn(async move {
                    debug!("Pid {:?} has confirmed pause", id);
                }),
                TaskMessage::ProcessStopped(id) => tokio::spawn(async move {
                    debug!("Pid {:?} has confirmed stop", id);
                }),
                TaskMessage::RequestLaunch(id) => tokio::spawn(async move {
                    let context = cloned_context.read().unwrap();
                    let service = context.get_service(id).unwrap();
                    let notify_type = context.is_notify_type(id);

                    // setup the socket to wait for this process to connect
                    if notify_type {
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            server::manage_a_service(tx, service, id).await;
                        });
                    }
                    if let Err(err) = context.launch_service(id) {
                        //TODO: Handle error
                        error!("Error launching service : {:?} due to {}", id, err);
                    } else {
                        debug!("launched service : {:?}", context.get_name(id));
                        tokio::spawn(async move {
                            let msg = if notify_type {
                                TaskMessage::ProcessLaunched(id)
                            } else {
                                TaskMessage::ProcessRunning(id)
                            };
                            if let Err(_) = tx.send(msg) {
                                panic!("Receiver dropped");
                            }
                        });
                    }
                }),
                TaskMessage::UeventReady => tokio::spawn(async move {
                    debug!("Uevent processing is ready");
                    if let Err(_) = sysfs_walker::launch_sysfs_walker() {
                        error!("Could not launch sysfs walker");
                    }
                }),
                TaskMessage::DeviceChanged(info) => tokio::spawn(async move {
                    match info {
                        DeviceChangeInfo::Added(dev) => {
                            info!("ADD:{}", dev);
                            if let Ok(index) = Context::do_unit(cloned_context, dev).await {
                                if let Err(_) = tx.send(TaskMessage::UnitSuccess(index)) {
                                    panic!("Receiver dropped");
                                }
                            }
                        }
                        DeviceChangeInfo::Removed(dev) => info!("REMOVE:{}", dev),
                        DeviceChangeInfo::Changed(dev) => info!("CHANGED:{}", dev),
                    }
                }),
                TaskMessage::UnitSuccess(unit_index) => tokio::spawn(async move {
                    debug!("Unit success for {:?}", unit_index);
                    let deps = cloned_context
                        .read()
                        .unwrap()
                        .get_immediate_dependant_services(unit_index);
                    for dep in deps {
                        debug!("Launching dep {:?}", dep);

                        if let Err(_) = tx.send(TaskMessage::RequestLaunch(dep)) {
                            panic!("Receiver dropped");
                        }
                    }
                }),
            };
        }
    });

    // use the main task as a signal receiver
    let mut stream = signal(SignalKind::child()).unwrap();
    loop {
        if let None = stream.recv().await {
            error!("Fatal: cannot receive signals anymore");
            return Err(std::io::Error::last_os_error());
        } else {
            //println!("Got signal Child\n");
            let (killed, _stopped, _continued) =
                cloned_context.write().unwrap().process_child_events();
            for pid in killed {
                if let Err(_) = tx.send(TaskMessage::ProcessExited(pid)) {
                    panic!("Receiver dropped.3");
                }
            }
        }
    }
}
