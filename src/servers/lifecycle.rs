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

use std::time::Duration;
use crate::common::*;
use crate::context::{ContextReference, ServiceIndex, RunningState};

use libinitception::application_interface::{self,ApplicationStatus};
//use crate::application::src_gen::application_interface_ttrpc;

use libinitception::{ApplicationManager,
    LifecycleServer,
    create_application_manager, 
    create_lifecycle_server,
};

use async_trait::async_trait;
use std::sync::Arc;

use tracing::{debug};
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

impl LifecycleServerImpl {
    pub fn new(
        context: ContextReference,
        tx: InnerTxReference,
        service_index: ServiceIndex,
    ) -> Self {
        LifecycleServerImpl {
            inner: Arc::new(std::sync::RwLock::new(Inner::new(
                context,
                tx,
                service_index,
            ))),
        }
    }
}

#[async_trait]
impl LifecycleServer for LifecycleServerImpl {
    async fn get_applications(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        _req: application_interface::GetApplicationsRequest,
    ) -> ttrpc::Result<application_interface::GetApplicationsResponse> {

        let inner = self.inner.read().unwrap();
        let context = inner.context.read().unwrap();
        let names = context.get_all_services();

        let mut response = application_interface::GetApplicationsResponse::default();
        response.set_name(names.into());

        Ok(response)

    }
    async fn get_application_status(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        req: application_interface::GetApplicationStatusRequest,
    ) -> ttrpc::Result<application_interface::GetApplicationStatusResponse> {

        let inner = self.inner.read().unwrap();
        let context = inner.context.read().unwrap();
        
        if let Some(status) = context.get_service_status(req.get_name()) {

            let mut response = application_interface::GetApplicationStatusResponse::default();
            match status {
                RunningState::Running => response.set_status(ApplicationStatus::Running),
                RunningState::Stopped => response.set_status(ApplicationStatus::Stopped),
                RunningState::Paused => response.set_status(ApplicationStatus::Paused),
                RunningState::WaitForConnect => response.set_status(ApplicationStatus::Running),
                //RunningState::Killed => response.set_status(ApplicationStatus::Stopped),
                RunningState::Unknown => response.set_status(ApplicationStatus::Stopped),
                //RunningState::Zombie => response.set_status(ApplicationStatus::Stopped),
            }
            
            Ok(response)

        } else {
            Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
                ttrpc::Code::NOT_FOUND,
                "/grpc.LifecycleServer/get_application_status is not supported".to_string(),
            )))
        }

        
    }
    async fn start_application(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        req: application_interface::StartApplicationRequest,
    ) -> ttrpc::Result<application_interface::StartApplicationResponse> {

        let timeout = Duration::from_millis(1000);
        let inner = self.inner.read().unwrap();
        
        let is_running = {
            let context = inner.context.read().unwrap();
            let index = context.get_service_index(req.get_name());
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
                let mut ret = application_interface::StartApplicationResponse::default();
                ret.set_status(application_interface::ReturnStatus::ERROR);
                return Ok(ret)
            }

            let mut ret = application_interface::StartApplicationResponse::default();

            //TODO: Wait until the process is actually launched
            if let Ok(recv) = rx.recv_timeout(timeout) {
                let status = match recv {
                    TaskReply::Ok => application_interface::ReturnStatus::OK,
                    TaskReply::Error => application_interface::ReturnStatus::ERROR,
                };
                ret.set_status(status);
                Ok(ret)
            } else {
                ret.set_status(application_interface::ReturnStatus::ERROR);
                Ok(ret)
            }
        } else {

            let mut ret = application_interface::StartApplicationResponse::default();
            ret.set_status(application_interface::ReturnStatus::OK);
            Ok(ret)
        }

    }
    async fn restart_application(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        _req: application_interface::RestartApplicationRequest,
    ) -> ttrpc::Result<application_interface::RestartApplicationResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
            ttrpc::Code::NOT_FOUND,
            "/grpc.LifecycleServer/restart_application is not supported".to_string(),
        )))
    }
    async fn pause_application(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        req: application_interface::PauseApplicationRequest,
    ) -> ttrpc::Result<application_interface::PauseApplicationResponse> {
        debug!("Pause application request for {}",req.get_name());
        let inner = self.inner.read().unwrap();
        let context = inner.context.read().unwrap();
        let index = context.get_service_index(req.get_name());
        let mut ret = application_interface::PauseApplicationResponse::default();
        if let Some(index) = index {
                 if context.is_running(index) {
                    let tx = inner.tx.lock().unwrap().clone();
                    let (sender, rx) = std::sync::mpsc::channel::<TaskReply>();
                    println!("Requesting pause for {}",req.get_name());
                    if let Err(_e) = tx.send(TaskMessage::RequestPause(index, Some(sender))) {
                        panic!("receiver dropped");
                    }

                   // wait for completion
                   if let Ok(recv) = rx.recv_timeout(Duration::from_millis(4000)) {
                    let status = match recv {
                        TaskReply::Ok => application_interface::ReturnStatus::OK,
                        TaskReply::Error => application_interface::ReturnStatus::ERROR,
                    };
                    ret.set_status(status);
                    Ok(ret)
                } else {
                    ret.set_status(application_interface::ReturnStatus::ERROR);
                    Ok(ret)
                }

                 } else {
                     Ok(application_interface::PauseApplicationResponse::default())
                 }
        }
        else {
            Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
                ttrpc::Code::NOT_FOUND,
                "/grpc.LifecycleServer/pause_application is not supported".to_string(),
            )))
        }
    }

    async fn resume_application(&self, _ctx: &::ttrpc::r#async::TtrpcContext, req: application_interface::ResumeApplicationRequest) -> ::ttrpc::Result<application_interface::ResumeApplicationResponse> {
        debug!("Resume application request for {}",req.get_name());
        let inner = self.inner.read().unwrap();
        let context = inner.context.read().unwrap();
        let index = context.get_service_index(req.get_name());
        let mut ret = application_interface::ResumeApplicationResponse::default();
        if let Some(index) = index {
                 if context.is_running(index) {
                    let tx = inner.tx.lock().unwrap().clone();
                    let (sender, rx) = std::sync::mpsc::channel::<TaskReply>();
                    debug!("Requesting resume for {}",req.get_name());
                    if let Err(_e) = tx.send(TaskMessage::RequestResume(index, Some(sender))) {
                        panic!("receiver dropped");
                    }

                   // wait for completion
                   if let Ok(recv) = rx.recv_timeout(Duration::from_millis(4000)) {
                    let status = match recv {
                        TaskReply::Ok => application_interface::ReturnStatus::OK,
                        TaskReply::Error => application_interface::ReturnStatus::ERROR,
                    };
                    ret.set_status(status);
                    Ok(ret)
                } else {
                    ret.set_status(application_interface::ReturnStatus::ERROR);
                    Ok(ret)
                }
                 } else {
                     Ok(application_interface::ResumeApplicationResponse::default())
                 }
        }
        else {
            Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
                ttrpc::Code::NOT_FOUND,
                "/grpc.LifecycleServer/resume_application is not supported".to_string(),
            )))
        }
    }

    async fn stop_application(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        req: application_interface::StopApplicationRequest,
    ) -> ttrpc::Result<application_interface::StopApplicationResponse> {

        debug!("Stop application request for {}",req.get_name());
        let inner = self.inner.read().unwrap();
        let context = inner.context.read().unwrap();
        let index = context.get_service_index(req.get_name());
        let mut ret = application_interface::StopApplicationResponse::default();
        if let Some(index) = index {
                 if context.is_running(index) {
                    let tx = inner.tx.lock().unwrap().clone();
                    let (sender, rx) = std::sync::mpsc::channel::<TaskReply>();
                    println!("Requesting stop for {}",req.get_name());
                    if let Err(_e) = tx.send(TaskMessage::RequestStop(index, Some(sender))) {
                        panic!("receiver dropped");
                    }

                   // wait for completion
                   if let Ok(recv) = rx.recv_timeout(Duration::from_millis(4000)) {
                    let status = match recv {
                        TaskReply::Ok => application_interface::ReturnStatus::OK,
                        TaskReply::Error => application_interface::ReturnStatus::ERROR,
                    };
                    ret.set_status(status);
                    Ok(ret)
                } else {
                    ret.set_status(application_interface::ReturnStatus::ERROR);
                    Ok(ret)
                }
                    //Ok(application_interface::StopApplicationResponse::default())
                 } else {
                     Ok(application_interface::StopApplicationResponse::default())
                 }
        }
        else {
            Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
                ttrpc::Code::NOT_FOUND,
                "/grpc.LifecycleServer/stop_application is not supported".to_string(),
            )))
        }
        
    }
    async fn prepare_system_freeze(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        _req: application_interface::PrepareFreezeRequest,
    ) -> ttrpc::Result<application_interface::PrepareFreezeResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
            ttrpc::Code::NOT_FOUND,
            "/grpc.LifecycleServer/prepare_system_freeze is not supported".to_string(),
        )))
    }
    async fn prepare_system_shutdown(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        _req: application_interface::PrepareShutdownRequest,
    ) -> ttrpc::Result<application_interface::PrepareShutdownResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
            ttrpc::Code::NOT_FOUND,
            "/grpc.LifecycleServer/prepare_system_shutdown is not supported".to_string(),
        )))
    }
    async fn shutdown_system(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        _req: application_interface::ShutdownRequest,
    ) -> ttrpc::Result<application_interface::ShutdownResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
            ttrpc::Code::NOT_FOUND,
            "/grpc.LifecycleServer/shutdown_system is not supported".to_string(),
        )))
    }
}
