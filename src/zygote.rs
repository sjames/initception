/// The zygote is responsible to setup the namespace and clone. The init process communicates with the
/// zygote using IPC
///
extern crate serde;

use crate::common::*;
use ipc_channel::ipc::{self, IpcOneShotServer, IpcReceiver, IpcReceiverSet, IpcSender, IpcSelectionResult};
use serde::{Deserialize, Serialize};

use nix::unistd::{fork, ForkResult};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecConfig {}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZygoteCommand {
    CmdLaunch(ExecConfig),
    RetLaunch(Result<u32, String>),
}

// The context that is retained by the Zygote client
pub struct ZycoteClientContext {
    zygote_pid: u32,
    tx: IpcSender<ZygoteCommand>,
    rx: IpcReceiver<ZygoteCommand>,
}

impl ZycoteClientContext {
    pub fn new(
        tx: IpcSender<ZygoteCommand>,
        rx: IpcReceiver<ZygoteCommand>,
    ) -> Result<ZycoteClientContext, &'static str> {
        Ok(ZycoteClientContext {
            zygote_pid: 0,
            tx: tx,
            rx: rx,
        })
    }
}

struct ZygoteContext {
    tx: IpcSender<ZygoteCommand>,
    rx: IpcReceiver<ZygoteCommand>,
}

impl ZygoteContext {
    fn new(
        tx: IpcSender<ZygoteCommand>,
        rx: IpcReceiver<ZygoteCommand>,
    ) -> Result<ZygoteContext, &'static str> {
        Ok(ZygoteContext { tx: tx, rx: rx })
    }
}

/// This call will spawn off a new process
pub fn run_zygote(context: &mut Context) -> Result<ZycoteClientContext, &'static str> {
    let (tx_to_zygote, rx_from_init) = ipc::channel::<ZygoteCommand>().unwrap();
    let (tx_to_init, rx_from_zygote) = ipc::channel::<ZygoteCommand>().unwrap();

    let mut zygote_client_context = ZycoteClientContext::new(tx_to_zygote, rx_from_zygote).unwrap();
    let mut zygote_context = ZygoteContext::new(tx_to_init, rx_from_init).unwrap();

    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {
            println!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );
            Ok(zygote_client_context)
        }
        Ok(ForkResult::Child) => {
            println!("I'm a new child process");
            match zygote_main(zygote_context) {
                Ok(_) => Err("Zygote exit OK"),
                Err(err) => Err("Zygote exit with failure"),
            }
        }
        Err(_) => {
            println!("Fork failed");
            Err("Fork failed")
        }
    }
}

fn handle_message(command : ZygoteCommand) -> Result<(),ZygoteError> {

   // match command {


//    }

    Err(ZygoteError::unknown_command)
}

fn zygote_main(context: ZygoteContext) -> Result<(), &'static str> {
    let mut rx_set = IpcReceiverSet::new().unwrap();
    let _rx_id = rx_set.add(context.rx).unwrap();

    loop {
        match rx_set.select() {
            Ok(res) => {
                for msg in res.into_iter() {
                    match msg {
                        IpcSelectionResult::MessageReceived(id, msg) => {
                            let command: ZygoteCommand = msg.to().unwrap();
                            handle_message(command).unwrap()
                        }
                        IpcSelectionResult::ChannelClosed(id) => {}
                    }
                }
            }
            Err(e) => {
                return Err("Select returned error");
            }
        }
    }

    //Ok(())
}

#[derive(Debug)]
pub enum ZygoteError {
    unknown_command,

}

impl fmt::Display for ZygoteError {
    fn fmt(&self, f:&mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            unknown_command => { write!(f,"Unknown Zygote command")}
        }
    }
}

impl std::error::Error for ZygoteError {
    fn description(&self) -> &str {
        match self {
            unknown_command => { "Unknown Zygote Command"}
        }
        
    }
}