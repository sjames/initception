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

use tokio::sync::oneshot::channel as oneshot_channel;
use tokio::sync::oneshot::Sender;
use tokio::time::timeout;

use tracing::{debug, error, info};

use crate::common::*;
use crate::context::{ContextReference, RuntimeEntityReference, ServiceIndex};

//use crate::application::src_gen::application_interface;
//use crate::application::src_gen::application_interface_ttrpc;

//use crate::application::src_gen::application_interface_ttrpc::{ApplicationServiceClient};
use crate::servers::application_client::ApplicationServiceWrapper;
use crate::servers::lifecycle::LifecycleControlServer;

//use libinitception::application_interface;
//use libinitception::{ApplicationManager, ApplicationServiceClient, LifecycleServer};

use async_trait::async_trait;

//use ttrpc::r#async::Client;
//use ttrpc::r#async::Server;

//use std::os::unix::io::FromRawFd;
use std::os::unix::io::IntoRawFd;
use std::sync::Arc;

use crate::servers::lifecycle::LifecycleServerImpl;
use libinitception::initrc;

use libinitception::app_manager_interface::*;
/*  New implementation for ServiceManager
*/
use someip::*;
use someip_derive::service_impl;

#[service_impl(ApplicationServer)]
struct ApplicationServerImpl {
    inner: InnerReference,
}

impl ApplicationServerImpl {
    fn new(
        context: ContextReference,
        tx: InnerTxReference,
        service_index: ServiceIndex,
        sender: Sender<()>,
    ) -> Self {
        ApplicationServerImpl {
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
impl ApplicationServer for ApplicationServerImpl {
    async fn heartbeat(&self, counter: u32) -> Result<(),ApplicationManagerError> {
        let inner = self.inner.write().unwrap();
        debug!("Heartbeat received from : {:?} counter {}", inner.service_index, counter);
        let service = inner.get_service().unwrap();
        let mut service = service.write().unwrap();
        service.record_watchdog();
        Ok(())
    }

    async fn statechanged(&self, running_state: RunningState) -> Result<(),ApplicationManagerError> {
        match running_state {
            RunningState::Paused => {
                let inner = self.inner.read().unwrap();
                let tx = inner.tx.lock().unwrap();

                if tx
                    .send(TaskMessage::ProcessPaused(inner.service_index, None))
                    .is_err()
                {
                    panic!("Receiver dropped");
                }
                Ok(())
            }
            RunningState::Running => {
                let mut inner = self.inner.write().unwrap();

                info!("Application is running");

                // send this once
                if let Some(tx) = inner.sender.take() {
                    if tx.send(()).is_err() {
                        panic!("Receiver dropped");
                    }
                }

                let tx = inner.tx.lock().unwrap();

                if tx
                    .send(TaskMessage::ProcessRunning(inner.service_index, None))
                    .is_err()
                {
                    panic!("Receiver dropped");
                }
                Ok(())
            }
            RunningState::Stopped => {
                let inner = self.inner.read().unwrap();
                let tx = inner.tx.lock().unwrap();

                if tx
                    .send(TaskMessage::ProcessStopped(inner.service_index, None))
                    .is_err()
                {
                    panic!("Receiver dropped");
                }
                Ok(())
            }
        }
    }

    async fn get_property(&self, key: String) -> Result<String,ApplicationManagerError> {
        let inner = self.inner.read().unwrap();
        
        let response = if let Ok(context) = inner.context.read() {
            if let Some(value) = context.get_property(&key) {
                Ok(value)
            } else {
                Err(ApplicationManagerError::NotFound)
            }
        } else {
            Err(ApplicationManagerError::Unknown)
        };
        response
    }

    async fn set_property(&self, key : String, value:String, options: SetPropertyOptions ) -> Result<(),ApplicationManagerError> {
        let inner = self.inner.read().unwrap();
        let context = inner.context.write();

        if let Ok(mut context) = context {
            // special handling for read only strings
            if key.starts_with("ro.") && context.contains_property(&key) {
                // return error if the key already exists and is read only
                // key exists, bail out
                error!("Attempt to set read-only key {} ignored", &key);
                Err(ApplicationManagerError::ReadOnly)
            } else {
                // ok not a read only property, lets continue
                let copy = value.clone();
                let key_copy = key.clone();
                if let Some(previous_value) = context.write_property_unchecked(key, value) {
                    if copy == previous_value {
                        debug!("Property written but value did not change.  No notifications");
                    } else {
                        //TODO: Change property notification
                        let tx = inner.tx.lock().unwrap();
                        if tx
                            .send(TaskMessage::PropertyChanged(
                                inner.service_index,
                                key_copy,
                                copy,
                            ))
                            .is_err()
                        {
                            panic!("Receiver dropped");
                        }
                    }
                }
                Ok(())
            }
        } else {
            Err(ApplicationManagerError::Unknown)
        }
      }
    
    async fn set_property_filter(&self, regex_filter: String) -> Result<(),ApplicationManagerError> {
        debug!("Adding property filter: {}", &regex_filter);
        let inner = self.inner.read().unwrap();

        if let Ok(context) = inner.context.read() {
            if let Some(service) = context.get_service(inner.service_index) {
                if let Ok(mut service) = service.write() {
                    if let Ok(_) = service.add_property_filter(&regex_filter) {
                        return Ok(())
                    }
                }
            }
        }
        Err(ApplicationManagerError::Unknown)
    }
}


/* End of new implementation
 */


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
            tx,
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
    let service = Arc::new(ApplicationServerImpl::new(
        orig_context.clone(),
        tx_arc,
        service_index,
        app_running_signal_tx,
    ));

    //let service = Arc::new(service);
    //let service = libinitception::create_application_manager(service);

    let mut app_server_handler = ApplicationServerImpl::create_server_request_handler(service);

    // If the service is a Lifecycle manager, then launch the server for it.
    let lifecycle_server = if let Some(initrc::ServiceType::LifecycleManager) = service_type {
        // channel to communicate with the initception main loop
        let tx_arc = std::sync::Arc::new(std::sync::Mutex::new(tx.clone()));

        let service = Arc::new(LifecycleControlServer::new(
            orig_context,
            tx_arc,
            service_index,
        ));

        let lifecycle_server_handler = LifecycleControlServer::create_server_request_handler(service);
        Some(lifecycle_server_handler)
    } else {
        None
    };

    let server = {
        let runtime_entity = context.read().unwrap().get_service(service_index).unwrap();
        let mut runtime_entity = runtime_entity.write();
        let runtime_entity = runtime_entity.as_deref_mut().unwrap();
        if let Some(fd) = runtime_entity.take_server_fd() {
            Some((
                {
                    if let Some(lifecycle_handler) = lifecycle_server {
                        app_server_handler.extend(lifecycle_handler);
                    } 

                    let handlers : Vec<(u16, Arc<dyn ServerRequestHandler>, u8, u32)> = app_server_handler.into_iter().map(|(a,h)|{

                        (id_of_service(a).unwrap(),h,1,0)
                
                    }).collect();
                    handlers
                },
                fd,
            ))
        } else {
            None
        }
    };

    if let Some((handlers, socket)) = server {

        tokio::spawn(async move {someip::Server::serve_uds(socket, &handlers).await});

        { // This block exists so that the locks are released.
            info!("Server started normally");
            //let tmp = context.read().unwrap().get_service(service_index).unwrap();
            let runtime_entity = context.read().unwrap();
            let runtime_entity = runtime_entity.get_service(service_index).unwrap();
            let runtime_entity = runtime_entity.write();
            if let Ok(mut runtime_entity) = runtime_entity {
                runtime_entity.set_terminate_signal_channel(app_server_terminate_tx)
            }
        }
    
        // wait for a "reasonable time" for the application to connect back.  The application must
        // load the server before connecting back so the client we launch here does not fail.
        let app_client_server = if timeout(Duration::from_millis(2000), app_running_signal_rx)
            .await
            .is_err()
        {
            error!("Application did not connect within 2000 milliseconds");
            None
        } else {
            // Application connected. Create the proxy and the server for this application
            debug!("Application connected. Creating Application proxy");
            let mut maybe_server = None;
            let runtime_entity = context.read().unwrap().get_service(service_index).unwrap();
            let mut runtime_entity = runtime_entity.write();
            let runtime_entity = runtime_entity.as_deref_mut().unwrap();

            if let Some(fd) = runtime_entity.take_client_fd() {

                let config = Configuration::default();

                let proxy = ApplicationControlProxy::new(id_of_service(ApplicationControlProxy::service_name()).unwrap(),1,config);
                let p = proxy.clone();

                //start processing the client
                tokio::spawn(async move {
                    ApplicationControlProxy::run_uds(p,fd).await}
                );

                // The term "server" here is confusing. It is a server for messages coming from within initception
                // but also a proxy to talk to the application.
                let (server, proxy) = ApplicationServiceWrapper::new_pair(
                    proxy
                );
                maybe_server = Some(server);
                runtime_entity.set_service_proxy(proxy);
            }
            maybe_server
        };

        if let Some(mut server) = app_client_server {
            tokio::spawn(async move { server.serve().await });
        }

        debug!("Application server waiting for termination signal");
        // cleanup when the application has terminated
        match app_server_terminate_rx.await {
            Ok(_) => {
                debug!("App server received termination message");
               
            }
            Err(_) => {
                panic!("App server terminate channel error");
            }
        }
    }
}
