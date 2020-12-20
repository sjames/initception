use crate::context::{ServiceIndex, ContextReference};
use crate::application::src_gen::{application_interface_ttrpc::ApplicationServiceClient, application_interface,application_interface_ttrpc};
use thiserror::Error;
use std::time::Duration;

use std::sync::{Arc, RwLock};

use tokio::sync::mpsc::{channel,Sender, Receiver};
use futures::TryFutureExt;

use tracing::{debug, info, warn};

use tokio::sync::oneshot::Sender as OneShotSender;
enum ApplicationRequest {
    Stop(OneShotSender<ApplicationResponse>, Duration),
    Pause(OneShotSender<ApplicationResponse>, Duration),
    Resume(OneShotSender<ApplicationResponse>, Duration),
    SessionChanged(OneShotSender<ApplicationResponse>, String),
    // An event with a key and value
    Event(OneShotSender<ApplicationResponse>,String, String),
    ServerQuit,
}

enum ApplicationResponse {
    Ok,
    Error(ServiceProxyError),
}


// Cloning the proxy is cheap as it only contains a sender
#[derive(Clone)]
pub struct ApplicationServiceProxy(Sender<ApplicationRequest>);

impl ApplicationServiceProxy {

    pub async fn stop(&mut self, timeout: Duration) -> Result<(),ServiceProxyError> {
        let (tx,rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if let Err(_) = self.0.send(ApplicationRequest::Stop(tx,timeout)).await {
            Err(ServiceProxyError::Disconnected)
        } else {
            if let Ok(ret) = rx.await {
                Ok(())
            } else {
                Err(ServiceProxyError::Disconnected)
            }
        }
    }
    pub async fn pause(&mut self, timeout: Duration) -> Result<(),ServiceProxyError> {
        let (tx,rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if let Err(_) = self.0.send(ApplicationRequest::Pause(tx,timeout)).await {
            Err(ServiceProxyError::Disconnected)
        } else {
            if let Ok(ret) = rx.await {
                Ok(())
            } else {
                Err(ServiceProxyError::Disconnected)
            }
        }
    }

    pub async fn resume(&mut self, timeout: Duration) -> Result<(),ServiceProxyError> {
        let (tx,rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if let Err(_) = self.0.send(ApplicationRequest::Resume(tx,timeout)).await {
            Err(ServiceProxyError::Disconnected)
        } else {
            if let Ok(ret) = rx.await {
                Ok(())
            } else {
                Err(ServiceProxyError::Disconnected)
            }
        }
    }

    pub async fn event(&mut self, key:&str, value:&str) -> Result<(),ServiceProxyError> {
        let (tx,rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if let Err(_) = self.0.send(ApplicationRequest::Event(tx,String::from(key), String::from(value))).await {
            Err(ServiceProxyError::Disconnected)
        } else {
            if let Ok(ret) = rx.await {
                Ok(())
            } else {
                Err(ServiceProxyError::Disconnected)
            }
        }
    }

    pub async fn session_changed(&mut self, session:&str) -> Result<(),ServiceProxyError> {
        let (tx,rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if let Err(_) = self.0.send(ApplicationRequest::SessionChanged(tx,String::from(session))).await {
            Err(ServiceProxyError::Disconnected)
        } else {
            if let Ok(ret) = rx.await {
                Ok(())
            } else {
                Err(ServiceProxyError::Disconnected)
            }
        }
    }

    pub async fn application_quit(&mut self) -> Result<(),ServiceProxyError> {
        let (tx,rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if let Err(_) = self.0.send(ApplicationRequest::ServerQuit).await {
            Err(ServiceProxyError::Disconnected)
        } else {
            if let Ok(ret) = rx.await {
                Ok(())
            } else {
                Err(ServiceProxyError::Disconnected)
            }
        }
    }
}

pub struct ApplicationServiceWrapper(ApplicationServiceClient,Receiver<ApplicationRequest>);

impl ApplicationServiceWrapper {
    pub fn new_pair(proxy: ApplicationServiceClient) -> (Self, ApplicationServiceProxy) {
        let (sender, rx) = tokio::sync::mpsc::channel::<ApplicationRequest>(1);
        (ApplicationServiceWrapper(proxy, rx), ApplicationServiceProxy(sender))
    }

    pub async fn serve(&mut self) {
        debug!("Started serving application");
        while let Some(msg) = self.1.recv().await {
            match msg {
                ApplicationRequest::Stop(tx,timeout) => {
                    if let Err(e) = self.stop(timeout).await {
                        let _e = tx.send(ApplicationResponse::Error(e));
                    } else {
                        let _e = tx.send(ApplicationResponse::Ok);
                    }
                },
                ApplicationRequest::Pause(tx, timeout) => {
                    if let Err(e) = self.pause(timeout).await {
                        let _e = tx.send(ApplicationResponse::Error(e));
                    } else {
                        let _e = tx.send(ApplicationResponse::Ok);
                    }
                }
                ApplicationRequest::Event(tx,key,value) => {
                    let timeout = Duration::from_millis(1000);
                    if let Err(e) = self.event((key,value), timeout).await {
                        let _e = tx.send(ApplicationResponse::Error(e));
                    } else {
                        let _e = tx.send(ApplicationResponse::Ok);
                    }
                },
                ApplicationRequest::Resume(tx, timeout) => {
                    let timeout = Duration::from_millis(1000);
                    if let Err(e) = self.resume(timeout).await {
                        let _e = tx.send(ApplicationResponse::Error(e));
                    } else {
                        let _e = tx.send(ApplicationResponse::Ok);
                    }
                },
                ApplicationRequest::SessionChanged(tx, session) => {
                    let timeout = Duration::from_millis(1000);
                    if let Err(e) = self.session_changed(session, timeout).await {
                        let _e = tx.send(ApplicationResponse::Error(e));
                    } else {
                        let _e = tx.send(ApplicationResponse::Ok);
                    }
                },
                ApplicationRequest::ServerQuit => {
                    break;
                }
            }
        }
        debug!("Application server stopped");
    }

    async fn pause(&mut self,timeout: Duration) -> Result<(),ServiceProxyError> {
        let req = application_interface::PauseRequest::new();
        if let Ok(res) = self.0.pause(&req, timeout.as_nanos() as i64).await {
            match res.status {
                application_interface::ReturnStatus::OK => {
                    Ok(())
                },
                _ => {
                    Err(res.status.into())
                }
            }
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }

    async fn resume(&mut self,  timeout: Duration) -> Result<(),ServiceProxyError> {

        let mut req = application_interface::ResumeRequest::new();
        req.set_timeout_ms(timeout.as_millis() as i32);
        if let Ok(res) = self.0.resume(&req, timeout.as_nanos() as i64).await {
            match res.status {
                application_interface::ReturnStatus::OK => {
                    Ok(())
                },
                _ => {
                    Err(res.status.into())
                }
            }
        } else {
            Err(ServiceProxyError::Disconnected)
        }

    }

    async fn session_changed(&mut self, session:String, timeout: Duration) -> Result<(),ServiceProxyError> {
        let mut req = application_interface::SessionChangedRequest::new();
        req.set_session_name(session);
        if let Ok(res) = self.0.session_changed(&req, timeout.as_nanos() as i64).await {
            match res.status {
                application_interface::ReturnStatus::OK => {
                    Ok(())
                },
                _ => {
                    Err(res.status.into())
                }
            }
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }

    async fn property_changed(&mut self,  prop: (&str,&str), timeout: Duration) -> Result<(),ServiceProxyError> {
        let mut req = application_interface::PropertyChangedRequest::new();
        req.set_key(String::from(prop.0));
        req.set_value(String::from(prop.1));

        if let Ok(res) = self.0.property_changed(&req, timeout.as_nanos() as i64).await {
            Ok(())
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }

    async fn event(&mut self,  prop: (String,String), timeout: Duration) -> Result<(),ServiceProxyError> {
            let mut req = application_interface::EventRequest::new();
            req.set_key(prop.0);
            req.set_value(prop.1);
            if let Ok(res) = self.0.event(&req, timeout.as_nanos() as i64).await {
                Ok(())
            } else {
                Err(ServiceProxyError::Disconnected)
            }
    }

    async fn stop(&mut self,  timeout: Duration) -> Result<(),ServiceProxyError> {
        let mut req = application_interface::StopRequest::new();
        req.set_timeout_ms(timeout.as_millis() as i32);
        
        if let Ok(res) = self.0.stop(&req, timeout.as_nanos() as i64).await {
            match res.status {
                application_interface::ReturnStatus::OK => {
                    Ok(())
                },
                _ => {
                    Err(res.status.into())
                }
            }
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }
}

#[derive(Error, Debug)]
pub enum ServiceProxyError {
    #[error("proxy disconnected disconnected")]
    Disconnected,
    #[error("Application action failed")]
    Failed,
    #[error("Application not ready")]
    Retry,
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    //#[error("invalid header (expected {expected:?}, found {found:?})")]
    //InvalidHeader {
    //    expected: String,
    //    found: String,
    //},
    #[error("unknown proxy error")]
    Unknown,
}

impl From<application_interface::ReturnStatus> for ServiceProxyError {
    fn from(status:application_interface::ReturnStatus) -> Self {
        match status {
            application_interface::ReturnStatus::ERROR => ServiceProxyError::Failed,
            application_interface::ReturnStatus::AGAIN => ServiceProxyError::Retry,
            application_interface::ReturnStatus::OK => { panic!("Unexpected as this is not an error")},
        }
    }
}