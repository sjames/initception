use std::time::Duration;


use tokio::stream::StreamExt;
use tokio::sync::oneshot::channel as oneshot_channel;
use tokio::sync::oneshot::Sender;
use tokio::time::timeout;

use tracing::{debug, error, info};


use crate::common::*;
use crate::context::{RuntimeEntityReference, ServiceIndex};


use crate::application::src_gen::application_interface;
use crate::application::src_gen::application_interface_ttrpc;
use crate::context::RuntimeEntity;

use crate::application::src_gen::application_interface_ttrpc::ApplicationServiceClient;
use async_trait::async_trait;
use ttrpc;
use ttrpc::r#async::Client;
use ttrpc::r#async::Server;

//use std::os::unix::io::FromRawFd;
use std::os::unix::io::IntoRawFd;
use std::sync::Arc;


pub struct LifecycleServerImpl {
    inner: InnerReference,
}

type InnerReference = std::sync::Arc<std::sync::RwLock<Inner>>;
type InnerTxReference = std::sync::Arc<std::sync::Mutex<SyncTxHandle>>;
struct Inner {
    spawnref: RuntimeEntityReference, // reference to the runtime entity for this server
    tx: InnerTxReference,
    service_index: ServiceIndex,
}

impl Inner {
    fn new(
        spawnref: RuntimeEntityReference,
        tx: InnerTxReference,
        service_index: ServiceIndex,
    ) -> Self {
        Inner {
            spawnref,
            tx,
            service_index,
        }
    }
}

impl LifecycleServerImpl {
    pub fn new(
        spawnref: RuntimeEntityReference,
        tx: InnerTxReference,
        service_index: ServiceIndex,
    ) -> Self {
        LifecycleServerImpl {
            inner: Arc::new(std::sync::RwLock::new(Inner::new(
                spawnref,
                tx,
                service_index,
            ))),
        }
    }
}

#[async_trait]
impl application_interface_ttrpc::LifecycleServer for LifecycleServerImpl {
    async fn get_applications(&self, _ctx: &ttrpc::r#async::TtrpcContext, _req: application_interface::GetApplicationsRequest) -> ttrpc::Result<application_interface::GetApplicationsResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/get_applications is not supported".to_string())))
    }
    async fn get_application_status(&self, _ctx: &ttrpc::r#async::TtrpcContext, _req: application_interface::GetApplicationStatusRequest) -> ttrpc::Result<application_interface::GetApplicationStatusResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/get_application_status is not supported".to_string())))
    }
    async fn start_application(&self, _ctx: &ttrpc::r#async::TtrpcContext, _req: application_interface::StartApplicationRequest) -> ttrpc::Result<application_interface::StartApplicationResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/start_application is not supported".to_string())))
    }
    async fn restart_application(&self, _ctx: &ttrpc::r#async::TtrpcContext, _req: application_interface::RestartApplicationRequest) -> ttrpc::Result<application_interface::RestartApplicationResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/restart_application is not supported".to_string())))
    }
    async fn pause_application(&self, _ctx: &ttrpc::r#async::TtrpcContext, _req: application_interface::PauseApplicationRequest) -> ttrpc::Result<application_interface::PauseApplicationResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/pause_application is not supported".to_string())))
    }
    async fn stop_application(&self, _ctx: &ttrpc::r#async::TtrpcContext, _req: application_interface::StopApplicationRequest) -> ttrpc::Result<application_interface::StopApplicationResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/stop_application is not supported".to_string())))
    }
    async fn prepare_system_freeze(&self, _ctx: &ttrpc::r#async::TtrpcContext, _req: application_interface::PrepareFreezeRequest) -> ttrpc::Result<application_interface::PrepareFreezeResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/prepare_system_freeze is not supported".to_string())))
    }
    async fn prepare_system_shutdown(&self, _ctx: &ttrpc::r#async::TtrpcContext, _req: application_interface::PrepareShutdownRequest) -> ttrpc::Result<application_interface::PrepareShutdownResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/prepare_system_shutdown is not supported".to_string())))
    }
    async fn shutdown_system(&self, _ctx: &ttrpc::r#async::TtrpcContext, _req: application_interface::ShutdownRequest) -> ttrpc::Result<application_interface::ShutdownResponse> {
        Err(ttrpc::Error::RpcStatus(ttrpc::get_status(ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/shutdown_system is not supported".to_string())))
    }
}
