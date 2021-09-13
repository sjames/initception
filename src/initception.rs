/*
    Copyright 2020 Sojan James
    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at
        http://www.apache.org/licenses/LICENSE-2.0
    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

extern crate tokio;
extern crate unshare;

use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio::signal::unix::{signal, SignalKind};
use tracing::{debug, error, info};

use crate::common::*;
use crate::context::{Context, ContextReference};
use crate::device;
use libinitception::config::ApplicationConfig;

use crate::server;
use crate::sysfs_walker;
use crate::ueventd;
use crate::zygote;

pub struct InitceptionConfig {
    /// Enable loopback network interface without
    /// explicit configuration.
    configure_lo : bool,
    /// perform early mounts. The initrd normally performs
    /// the early mounts so this is disabled by default.
    early_mounts : bool,
    /// Path to the data partition if present. If a data
    /// partition is present, a directory will be created in it
    /// for each application and the application home directory will be 
    /// set to it
    data_root : PathBuf,
}

impl Default for InitceptionConfig {
    fn default() -> Self {
        Self { configure_lo: false, early_mounts : false, data_root: PathBuf::from_str("/").unwrap() }
    }
}

impl InitceptionConfig {
    pub fn enable_lo(mut self, en: bool) -> Self {
        self.configure_lo = en;
        self
    }
    pub fn enable_early_mounts(mut self, en: bool) -> Self {
        self.early_mounts = en;
        self
    }
    pub fn set_data_mount(mut self, data_root: PathBuf) -> Self {
        self.data_root = data_root;
        self
    }
    pub fn data_mount(&self) -> &std::path::Path {
        self.data_root.as_path()
    }
}

pub fn initception_main(config: InitceptionConfig) -> Result<(), Box<dyn Error>> {

    if config.early_mounts {
        if let Err(e) = device::mount_basics() {
            error!("Unable to mount basics");
            return Err(e);
        }

        if let Err(e) = device::make_basic_devices() {
            error!("Unable to make basic devices");
            return Err(e);
        }
    }


    if let Err(e) = zygote::launch_zygote() {
        error!("Error launching zygote");
        return Err(e);
    }

    let context = Context::create_context(config).unwrap();
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
    config: InitceptionConfig,
) -> Result<(), Box<dyn Error>> {

    if config.early_mounts {
        if let Err(e) = device::mount_basics() {
            error!("Unable to mount basics");
            return Err(e);
        }
        if let Err(e) = device::make_basic_devices() {
            error!("Unable to make basic devices");
            return Err(e);
        }
    }
    

    // Create initial context by loading from config file.
    let mut context = Context::create_context(config).unwrap();
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

/*
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
*/

#[tokio::main]
async fn init_async_main(context: ContextReference) -> Result<(), std::io::Error> {
    let (tx_orig, rx) = std::sync::mpsc::channel::<TaskMessage>();

    {
        // get the list of services that can be started immediately
        let initial_services = context.read().unwrap().get_initial_services();
        println!("Initial services are: {:?}",&initial_services);

        let tx = tx_orig.clone();
        debug!("async main started");

        { // This block is intentional to release the read lock on the context immediately.
            let context = context.write().unwrap();
            let config = context.config();
            if config.configure_lo {
                if tx.send(TaskMessage::ConfigureNetworkLoopback).is_err() {
                    panic!("Receiver dropped when configuring network");
                }
            }
        

            for service_idx in initial_services {
                context.mark_launch_requested(service_idx);
                if tx
                    .send(TaskMessage::RequestLaunch(service_idx, None))
                    .is_err()
                {
                    panic!("Receiver dropped");
                }
            }
        }
    }

    // Spawn the task for uevent processing
    let tx = tx_orig.clone();
    // uevent calls blocking functions.
    tokio::spawn(async move {ueventd::uevent_main(tx).await});

    let cloned_context = context.clone();
    // Needed for the signal function below
    let tx = tx_orig.clone();

    // This is the main dispatch function for initception
    tokio::task::spawn_blocking(move || {
        while let Ok(msg) = rx.recv() {
            let cloned_context = context.clone();
            let _mount_context = context.clone();
            let tx = tx_orig.clone();
            match msg {
                TaskMessage::ConfigureNetworkLoopback => tokio::spawn(async move {
                    debug!("Configure Loopback network interface");
                    let ip = ipnetwork::IpNetwork::V4("127.0.0.1".parse().unwrap());
                    let _ = crate::network::configure_network_interface(ip, String::from("lo"));
                    debug!("Loopback network set up");
                }),
                TaskMessage::ProcessRunning(id, _notify) => tokio::spawn(async move {
                    debug!("Process Running {:?}", id);
                    let deps = cloned_context.read().unwrap().get_immediate_dependants(id);
                    for dep in deps {
                        debug!("Launching dep {:?}", dep);
                        if !cloned_context.read().unwrap().is_launched(id) {
                            cloned_context.write().unwrap().mark_launch_requested(id);
                            if tx.send(TaskMessage::RequestLaunch(dep, None)).is_err() {
                                panic!("Receiver dropped");
                            }
                        }
                    }
                }),
                TaskMessage::ProcessLaunched(id, _notify) => tokio::spawn(async move {
                    debug!("Process launched {:?}", id);
                    let deps = cloned_context
                        .read()
                        .unwrap()
                        .get_immediate_dependant_services(id);
                    for dep in deps {
                        debug!("Launching dep {:?}", dep);
                        if !cloned_context.read().unwrap().is_launched(dep) {
                            cloned_context.write().unwrap().mark_launch_requested(dep);
                            if tx.send(TaskMessage::RequestLaunch(dep, None)).is_err() {
                                panic!("Receiver dropped");
                            }
                        }
                    }
                }),
                TaskMessage::ProcessExited(id, _notify) => tokio::spawn(async move {
                    debug!("Pid {:?} has exited", id);
                    if let Some(context) = cloned_context.write().unwrap().get_service(id) {
                        context.write().unwrap().cleanup_resources();
                    }

                    if let Some(time_ms) = cloned_context.read().unwrap().check_restart(id) {
                        if !cloned_context.read().unwrap().is_launched(id) {
                            cloned_context.write().unwrap().mark_launch_requested(id);
                            tokio::spawn(async move {
                                tokio::time::sleep(Duration::from_millis(time_ms as u64)).await;
                                if tx.send(TaskMessage::RequestLaunch(id, None)).is_err() {
                                    panic!("Receiver dropped");
                                }
                            });
                        }
                    }
                }),
                TaskMessage::ProcessPaused(id, _notify) => tokio::spawn(async move {
                    debug!("Pid {:?} has confirmed pause", id);
                }),
                TaskMessage::ProcessStopped(id, _notify) => tokio::spawn(async move {
                    debug!("Pid {:?} has confirmed stop", id);
                }),
                TaskMessage::RequestResume(id, notify) => {
                    let context = context.clone();
                    debug!("TASKMESSAGE:RequestResume {:?}", id);

                    tokio::spawn(async move {
                        debug!("Request stop for {:?}", id);

                        if let Err(ret) = crate::context::resume_service(context, id).await {
                            error!("Resume service failed : {}", ret);
                        } else {
                            debug!("Success resuming service");
                        }

                        if let Some(notify) = notify {
                            let _ = notify.send(TaskReply::Ok);
                        }
                    })
                }
                TaskMessage::RequestPause(id, notify) => {
                    let context = context.clone();
                    debug!("TASKMESSAGE:RequestPause {:?}", id);

                    tokio::spawn(async move {
                        debug!("Request stop for {:?}", id);

                        if let Err(ret) = crate::context::pause_service(context, id).await {
                            error!("Pause service failed : {}", ret);
                        } else {
                            debug!("Success pausing service");
                        }

                        if let Some(notify) = notify {
                            let _ = notify.send(TaskReply::Ok);
                        }
                    })
                }
                TaskMessage::RequestStop(id, notify) => {
                    let context = context.clone();
                    debug!("TASKMESSAGE:RequestStop {:?}", id);

                    tokio::spawn(async move {
                        debug!("Request stop for {:?}", id);
                        let _timeout = std::time::Duration::from_secs(2);

                        if let Err(ret) = crate::context::kill_service(context, id).await {
                            error!("Kill service failed : {}", ret);
                        } else {
                            debug!("Success killing service");
                        }

                        if let Some(notify) = notify {
                            let _ = notify.send(TaskReply::Ok);
                        }
                    })
                }
                TaskMessage::RequestLaunch(id, mut notify) => {
                    let server_context = context.clone();
                    let (notify_type, name) = {
                        let context = cloned_context.read().unwrap();
                        (context.is_notify_type(id), context.get_name(id))
                    };
                    debug!("{} is notify_type:{}",name.as_ref().unwrap(),notify_type);
                    let context = cloned_context.clone();

                    tokio::spawn(async move {
                        //let service = context.get_service(id).unwrap();
                        // setup the socket to wait for this process to connect
                        if notify_type {
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                server::manage_a_service(server_context, tx, id).await;
                            });
                        }
                        if let Err(err) = context.read().unwrap().launch_service(id) {
                            //TODO: Handle error
                            error!("Error launching service : {:?} due to {}", id, err);
                        } else {
                            debug!("launched service : {:?}", name.unwrap(),);

                            // notify success if someone has requested for it
                            if let Some(tx) = notify.take() {
                                let _ = tx.send(TaskReply::Ok);
                            }
                            // We wait for notification from an application
                            // before we mark it as running.

                            let msg = if notify_type {
                                TaskMessage::ProcessLaunched(id, notify)
                            } else {
                                TaskMessage::ProcessRunning(id, notify)
                            };
                            if tx.send(msg).is_err() {
                                panic!("Receiver dropped");
                            }
                        }
                    })
                }
                TaskMessage::UeventReady => tokio::spawn(async move {
                    debug!("Uevent processing is ready");
                    if sysfs_walker::launch_sysfs_walker().is_err() {
                        error!("Could not launch sysfs walker");
                    }
                }),
                TaskMessage::DeviceChanged(info) => {
                    let context = cloned_context.clone();
                    tokio::spawn(async move {
                        match info {
                            DeviceChangeInfo::Added(dev) => {
                                info!("ADD:{}", dev);
                                if let Ok(index) = Context::do_unit(context.clone(), dev).await {
                                    if tx.send(TaskMessage::UnitSuccess(index)).is_err() {
                                        panic!("Receiver dropped");
                                    }
                                }
                            }
                            DeviceChangeInfo::Removed(dev) => info!("REMOVE:{}", dev),
                            DeviceChangeInfo::Changed(dev) => info!("CHANGED:{}", dev),
                        }
                    })
                }
                TaskMessage::UnitSuccess(unit_index) => tokio::spawn(async move {
                    debug!("Unit success for {:?}", unit_index);
                    let deps = cloned_context
                        .read()
                        .unwrap()
                        .get_immediate_dependant_services(unit_index);
                    for dep in deps {
                        debug!("Launching dep {:?}", dep);

                        if !cloned_context.read().unwrap().is_launched(dep) {
                            cloned_context.write().unwrap().mark_launch_requested(dep);
                            if tx.send(TaskMessage::RequestLaunch(dep, None)).is_err() {
                                panic!("Receiver dropped");
                            }
                        } else {
                            println!("{:?} is already running",dep );
                        }

                        
                    }
                }),
                TaskMessage::PropertyChanged(_unit_index, key, value) => tokio::spawn(async move {
                    debug!("Property changed  {}:{}", &key, &value);
                }),
            };
        }
    });

    // use the main task as a signal receiver
    let mut stream = signal(SignalKind::child()).unwrap();
    loop {
        if stream.recv().await.is_none() {
            error!("Fatal: cannot receive signals anymore");
            return Err(std::io::Error::last_os_error());
        } else {
            //println!("Got signal Child\n");
            let (killed, _stopped, _continued) =
                cloned_context.write().unwrap().process_child_events();
            for pid in killed {
                if tx.send(TaskMessage::ProcessExited(pid, None)).is_err() {
                    panic!("Receiver dropped");
                }
            }
        }
    }
}
