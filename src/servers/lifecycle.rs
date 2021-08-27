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

use crate::common::*;
use crate::context::{ContextReference, RunningState, ServiceIndex};
use std::time::Duration;

//use libinitception::application_interface::{self, ApplicationStatus};
//use crate::application::src_gen::application_interface_ttrpc;



use async_trait::async_trait;
use std::sync::Arc;

use tracing::debug;

/*
 New implementation for LifecycleServer
*/
use libinitception::app_interface::*;
use someip::*;
use someip_derive::service_impl;

#[service_impl(LifecycleControl)]
pub struct LifecycleControlServer {
    inner: InnerReference,
}

impl LifecycleControlServer {
    pub fn new(
        context: ContextReference,
        tx: InnerTxReference,
        service_index: ServiceIndex,
    ) -> Self {
        LifecycleControlServer {
            inner: Arc::new(std::sync::RwLock::new(Inner::new(
                context,
                tx,
                service_index,
            ))),
        }
    }
}

#[async_trait]
impl LifecycleControl for LifecycleControlServer {
    async fn get_applications(&self) -> Result<Vec<String>,LifecycleControlError> {
        let inner = self.inner.read().unwrap();
        let context = inner.context.read().unwrap();
        let names = context.get_all_services();
        Ok(names)
    }

    async fn get_application_status(&self,name: String) -> Result<libinitception::app_interface::ApplicationStatus,LifecycleControlError> {
        let inner = self.inner.read().unwrap();
        let context = inner.context.read().unwrap();

        if let Some(status) = context.get_service_status(&name) {
            let response = match status {
                RunningState::Running => libinitception::app_interface::ApplicationStatus::Running,
                RunningState::Stopped => libinitception::app_interface::ApplicationStatus::Stopped,
                RunningState::Paused => libinitception::app_interface::ApplicationStatus::Paused,
                RunningState::WaitForConnect => libinitception::app_interface::ApplicationStatus::Running,
                //RunningState::Killed => response.set_status(ApplicationStatus::Stopped),
                RunningState::Unknown => libinitception::app_interface::ApplicationStatus::Stopped,
                //RunningState::Zombie => response.set_status(ApplicationStatus::Stopped),
            };
            Ok(response)
        } else {
            Err(libinitception::app_interface::LifecycleControlError::Unknown)
        }
    }

    async fn start_application(&self,name: String) -> Result<std::time::Duration,LifecycleControlError> {
        let timeout = Duration::from_millis(1000);
        let inner = self.inner.read().unwrap();
        let now = std::time::Instant::now();

        let is_running = {
            let context = inner.context.read().unwrap();
            let index = context.get_service_index(&name);
            let tx = inner.tx.lock().unwrap().clone();
            if let Some(index) = index {
                Some((context.is_running(index), index, tx))
            } else {
                None
            }
        };

        if let Some((is_running, index, tx)) = is_running {
            let (sender, rx) = std::sync::mpsc::channel::<TaskReply>();
            if !is_running {
                // let sender = sender.c
                if let Err(_e) = tx.send(TaskMessage::RequestLaunch(index, Some(sender))) {
                    panic!("receiver dropped");
                }
            } else {
                return Err(LifecycleControlError::AlreadyRunning);
            }

            //TODO: Wait until the process is actually launched
            if let Ok(recv) = rx.recv_timeout(timeout) {
                let status = match recv {
                    TaskReply::Ok => Ok(std::time::Instant::now() - now),
                    TaskReply::Error => Err(LifecycleControlError::Unknown),
                };
                status
            } else {
                Err(LifecycleControlError::Unknown)
            }
        } else {
            Err(LifecycleControlError::Unknown)
        }
    }

    async fn restart_application(&self,name: String) -> Result<std::time::Duration,LifecycleControlError> {
        Err(LifecycleControlError::Unsupported)
    }

    async fn pause_application(&self,name: String) -> Result<std::time::Duration,LifecycleControlError> {
        debug!("Pause application request for {}", &name);
        let start_time = std::time::Instant::now();
        let inner = self.inner.read().unwrap();
        let context = inner.context.read().unwrap();
        let index = context.get_service_index(&name);
        
        if let Some(index) = index {
            if context.is_running(index) {
                let tx = inner.tx.lock().unwrap().clone();
                let (sender, rx) = std::sync::mpsc::channel::<TaskReply>();
                println!("Requesting pause for {}", &name);
                if let Err(_e) = tx.send(TaskMessage::RequestPause(index, Some(sender))) {
                    panic!("receiver dropped");
                }

                // wait for completion
                if let Ok(recv) = rx.recv_timeout(Duration::from_millis(4000)) {
                    let status = match recv {
                        TaskReply::Ok => Ok(std::time::Instant::now() - start_time),
                        TaskReply::Error => Err(LifecycleControlError::ControlError),
                    };
                    status
                } else {
                    Err(LifecycleControlError::Unknown)
                }
            } else {
                Ok(std::time::Duration::from_millis(0))
            }
        } else {
            Err(LifecycleControlError::Unknown)
        }
    }

    async fn resume_application(&self,name: String) -> Result<std::time::Duration,LifecycleControlError> {
        debug!("Resume application request for {}", &name);
        let inner = self.inner.read().unwrap();
        let context = inner.context.read().unwrap();
        let index = context.get_service_index(&name);
        let start_time = std::time::Instant::now();

        if let Some(index) = index {
            if context.is_running(index) {
                let tx = inner.tx.lock().unwrap().clone();
                let (sender, rx) = std::sync::mpsc::channel::<TaskReply>();
                debug!("Requesting resume for {}", &name);
                if let Err(_e) = tx.send(TaskMessage::RequestResume(index, Some(sender))) {
                    panic!("receiver dropped");
                }

                // wait for completion
                if let Ok(recv) = rx.recv_timeout(Duration::from_millis(4000)) {
                    let status = match recv {
                        TaskReply::Ok => Ok(std::time::Instant::now() - start_time),
                        TaskReply::Error => Err(LifecycleControlError::ControlError),
                    };
                    status
                } else {
                    Err(LifecycleControlError::Unknown)
                }
            } else {
                Ok(std::time::Duration::from_millis(0))
            }
        } else {
            Err(LifecycleControlError::Unknown)
        }
    }

    async fn stop_application(&self,name: String) -> Result<std::time::Duration,LifecycleControlError> {
        debug!("Stop application request for {}", &name);
        let inner = self.inner.read().unwrap();
        let context = inner.context.read().unwrap();
        let index = context.get_service_index(&name);
        let start_time = std::time::Instant::now();
        
        if let Some(index) = index {
            if context.is_running(index) {
                let tx = inner.tx.lock().unwrap().clone();
                let (sender, rx) = std::sync::mpsc::channel::<TaskReply>();
                println!("Requesting stop for {}", &name);
                if let Err(_e) = tx.send(TaskMessage::RequestStop(index, Some(sender))) {
                    panic!("receiver dropped");
                }

                // wait for completion
                if let Ok(recv) = rx.recv_timeout(Duration::from_millis(4000)) {
                    let status = match recv {
                        TaskReply::Ok => Ok(std::time::Instant::now() - start_time),
                        TaskReply::Error => Err(LifecycleControlError::ControlError),
                    };
                    status
                } else {
                    Err(LifecycleControlError::Unknown)
                }
            } else {
                Ok(std::time::Duration::from_millis(0))
            }
        } else {
            Err(LifecycleControlError::Unknown)
        }
    }

    async fn prepare_system_freeze(&self, shutdown_type: ShutdownType) -> Result<(),LifecycleControlError> {
        Err(LifecycleControlError::Unsupported)
    }

    async fn prepare_system_shutdown(&self, shutdown_type: ShutdownType) -> Result<(),LifecycleControlError> {
        Err(LifecycleControlError::Unsupported)
    }

    async fn shutdown_system(&self, shutdown_type: ShutdownType) -> Result<(),LifecycleControlError> {
        Err(LifecycleControlError::Unsupported)
    }
}


/*
End new implementation
 */



pub struct LifecycleServerImpl {
    inner: InnerReference,
}

type InnerReference = std::sync::Arc<std::sync::RwLock<Inner>>;
type InnerTxReference = std::sync::Arc<std::sync::Mutex<SyncTxHandle>>;
struct Inner {
    context: ContextReference, // reference to the runtime entity for this server
    tx: InnerTxReference,
    service_index: ServiceIndex,
}

impl Inner {
    fn new(context: ContextReference, tx: InnerTxReference, service_index: ServiceIndex) -> Self {
        Inner {
            context,
            tx,
            service_index,
        }
    }
}



