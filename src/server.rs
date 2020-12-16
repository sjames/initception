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


use tokio::stream::StreamExt;
use tokio::sync::oneshot::channel as oneshot_channel;
use tokio::sync::oneshot::Sender;
use tokio::time::timeout;

use tracing::{debug, error, info};


use crate::common::*;
use crate::context::{RuntimeEntityReference, ServiceIndex, ContextReference};


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

use crate::servers::lifecycle::LifecycleServerImpl;
use crate::initrc;

struct ServiceManager {
    inner: InnerReference,
}

type InnerReference = std::sync::Arc<std::sync::RwLock<Inner>>;
type InnerTxReference = std::sync::Arc<std::sync::Mutex<SyncTxHandle>>;
struct Inner {
    context: ContextReference, // reference to the runtime entity for this server
    tx: InnerTxReference,
    service_index: ServiceIndex,
    sender: Option<Sender<()>>,
}

impl Inner {
    fn new(
        context: ContextReference,
        tx: InnerTxReference,
        service_index: ServiceIndex,
        sender: Sender<()>,
    ) -> Self {
        Inner {
            context,
            tx: tx,
            service_index,
            sender: Some(sender),
        }
    }

    fn get_service(&self) -> Option<RuntimeEntityReference> {
        self.context.read().unwrap().get_service(self.service_index)
    }
}

impl ServiceManager {
    fn new(
        context: ContextReference,
        tx: InnerTxReference,
        service_index: ServiceIndex,
        sender: Sender<()>,
    ) -> Self {
        ServiceManager {
            inner: Arc::new(std::sync::RwLock::new(Inner::new(
                context,
                tx,
                service_index,
                sender,
            ))),
        }
    }
}

#[async_trait]
impl application_interface_ttrpc::ApplicationManager for ServiceManager {
    async fn heartbeat(
        &self,
        _ctx: &::ttrpc::r#async::TtrpcContext,
        _req: application_interface::HeartbeatRequest,
    ) -> ::ttrpc::Result<application_interface::HeartbeatResponse> {
        let inner = self.inner.write().unwrap();
        info!("Heartbeat received from : {:?}", inner.service_index);

        let service = inner.get_service().unwrap();
        let mut service = service.write().unwrap();
        service.record_watchdog();

        Ok(application_interface::HeartbeatResponse::default())
    }
    async fn statechanged(
        &self,
        _ctx: &::ttrpc::r#async::TtrpcContext,
        _req: application_interface::StateChangedRequest,
    ) -> ::ttrpc::Result<application_interface::StateChangedResponse> {
        match _req.state {
            application_interface::StateChangedRequest_State::Paused => {
                let inner = self.inner.read().unwrap();
                let tx = inner.tx.lock().unwrap();

                if let Err(_) = tx.send(TaskMessage::ProcessPaused(inner.service_index)) {
                    panic!("Receiver dropped");
                }
                Ok(application_interface::StateChangedResponse::default())
            }
            application_interface::StateChangedRequest_State::Running => {
                let mut inner = self.inner.write().unwrap();

                info!("Application is running");

                // send this once
                if let Some(tx) = inner.sender.take() {
                    if let Err(_) = tx.send(()) {
                        panic!("Receiver dropped");
                    }
                }

                let tx = inner.tx.lock().unwrap();

                if let Err(_) = tx.send(TaskMessage::ProcessRunning(inner.service_index)) {
                    panic!("Receiver dropped");
                }
                Ok(application_interface::StateChangedResponse::default())
            }
            application_interface::StateChangedRequest_State::Stopped => {
                let inner = self.inner.read().unwrap();
                let tx = inner.tx.lock().unwrap();

                if let Err(_) = tx.send(TaskMessage::ProcessStopped(inner.service_index)) {
                    panic!("Receiver dropped");
                }
                Ok(application_interface::StateChangedResponse::default())
            }
        }
    }
}

/// spawn a server to handle a service. The tx handle is used to send back messages to
/// the main task. The stream is used to communicate with the process.
/// the spanwnref gives you a shared reference to the launched service context
pub async fn manage_a_service(
    orig_context: ContextReference,
    tx: SyncTxHandle,
    service_index: ServiceIndex,
) {
    info!("App manager server");

    let context = orig_context.clone();
   
    let service_type = context.read().unwrap().get_service(service_index).unwrap();
    let service_type = service_type.read().unwrap().is_service();

    let tx_arc = std::sync::Arc::new(std::sync::Mutex::new(tx.clone()));

    let (app_running_signal_tx, app_running_signal_rx) = oneshot_channel::<()>();

    // this channel is used to signal termination of the server
    let (app_server_terminate_tx, app_server_terminate_rx) = oneshot_channel::<()>();
    let service = Box::new(ServiceManager::new(
        orig_context.clone(),
        tx_arc,
        service_index,
        app_running_signal_tx,
    )) as Box<dyn application_interface_ttrpc::ApplicationManager + Send + Sync>;

    let service = Arc::new(service);
    let service = application_interface_ttrpc::create_application_manager(service);

    // If the service is a Lifecycle manager, then launch the server for it.
    let lifecycle_server = if let Some(initrc::ServiceType::LifecycleManager) = service_type {
        // channel to communicate with the initception main loop
        let tx_arc = std::sync::Arc::new(std::sync::Mutex::new(tx.clone()));

        let service = Box::new(LifecycleServerImpl::new(
            orig_context,
            tx_arc,
            service_index,
        )) as Box<dyn application_interface_ttrpc::LifecycleServer + Send + Sync>;
        let service = Arc::new(service);
        let service = application_interface_ttrpc::create_lifecycle_server(service);
        Some(service)
    } else { None};

    let mut server =  {
        let runtime_entity = context.read().unwrap().get_service(service_index).unwrap();
        let mut runtime_entity = runtime_entity.write();
        let runtime_entity = runtime_entity.as_deref_mut().unwrap();
        if let Some(fd) = runtime_entity.take_server_fd() {
                Some((
                    {
                        let server = Server::new().register_service(service).set_domain_unix();
                        // if lifecycle server exists, also register its methods
                        if let Some(lifecycle_server) = lifecycle_server {
                             server.register_service(lifecycle_server)
                        } else {
                            server
                        }
                        
                    },
                    fd,
                ))
        } else {
            None
        }

    };

    if let Some((mut server,socket)) = server
    {
        match server.start_single(socket).await {
            Ok(_) => {
                info!("Server started normally");
                //let tmp = context.read().unwrap().get_service(service_index).unwrap();
                let runtime_entity = context.read().unwrap();
                let runtime_entity = runtime_entity.get_service(service_index).unwrap();
                let runtime_entity = runtime_entity.write();
                if let Ok(mut runtime_entity)  = runtime_entity {
                    runtime_entity.set_terminate_signal_channel(app_server_terminate_tx)
                }
            }
            Err(e) => {
                panic!("Server not created: {}", e);
            }
        }

        // wait for a "reasonable time" for the application to connect back.  The application must
    // load the server before connecting back so the client we launch here does not fail.
    if let Err(_) = timeout(Duration::from_millis(2000), app_running_signal_rx).await {
        error!("Application did not connect within 2000 milliseconds");
    } else {
        // Application connected. Create the proxy
        let runtime_entity = context.read().unwrap().get_service(service_index).unwrap();
        let mut runtime_entity = runtime_entity.write();
        let runtime_entity = runtime_entity.as_deref_mut().unwrap();

        if let Some(fd) = runtime_entity.take_server_fd() {
            runtime_entity.set_service_proxy(ApplicationServiceClient::new(Client::new(fd.into_raw_fd())));
        }

    }

    // cleanup when the application has terminated
    match app_server_terminate_rx.await {
        Ok(_) => {
            debug!("App server received termination message");
            if let Err(_) = server.shutdown().await {
                error!("App server shutdown failure");
            }
        }
        Err(_) => {
            panic!("App server terminate channel error");
        }
    }




    }

    
}
