//use crate::application::src_gen::{application_interface_ttrpc::ApplicationServiceClient, application_interface};
use std::time::Duration;
use someip::CallProperties;
use thiserror::Error;

use libinitception::application_interface;
use libinitception::ApplicationServiceClient;
use libinitception::app_manager_interface::{*};

use tokio::sync::mpsc::{Receiver, Sender};

use tracing::debug;

use tokio::sync::oneshot::Sender as OneShotSender;

enum ApplicationRequest {
    Stop(OneShotSender<ApplicationResponse>, Duration),
    Pause(OneShotSender<ApplicationResponse>, Duration),
    Resume(OneShotSender<ApplicationResponse>, Duration),
    SessionChanged(OneShotSender<ApplicationResponse>, String),
    // An event with a key and value
    Event(OneShotSender<ApplicationResponse>, String, String),
    Property(OneShotSender<ApplicationResponse>, String, String),
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
    pub async fn stop(&mut self, timeout: Duration) -> Result<(), ServiceProxyError> {
        let (tx, rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if self
            .0
            .send(ApplicationRequest::Stop(tx, timeout))
            .await
            .is_err()
        {
            Err(ServiceProxyError::Disconnected)
        } else if let Ok(_ret) = rx.await {
            Ok(())
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }
    pub async fn pause(&mut self, timeout: Duration) -> Result<(), ServiceProxyError> {
        let (tx, rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if self
            .0
            .send(ApplicationRequest::Pause(tx, timeout))
            .await
            .is_err()
        {
            Err(ServiceProxyError::Disconnected)
        } else if let Ok(_ret) = rx.await {
            Ok(())
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }

    pub async fn resume(&mut self, timeout: Duration) -> Result<(), ServiceProxyError> {
        let (tx, rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if self
            .0
            .send(ApplicationRequest::Resume(tx, timeout))
            .await
            .is_err()
        {
            Err(ServiceProxyError::Disconnected)
        } else if let Ok(_ret) = rx.await {
            Ok(())
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }

    pub async fn event(&mut self, key: &str, value: &str) -> Result<(), ServiceProxyError> {
        let (tx, rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if self
            .0
            .send(ApplicationRequest::Event(
                tx,
                String::from(key),
                String::from(value),
            ))
            .await
            .is_err()
        {
            Err(ServiceProxyError::Disconnected)
        } else if let Ok(_ret) = rx.await {
            Ok(())
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }

    pub async fn property_changed(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), ServiceProxyError> {
        let (tx, rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if self
            .0
            .send(ApplicationRequest::Property(
                tx,
                String::from(key),
                String::from(value),
            ))
            .await
            .is_err()
        {
            Err(ServiceProxyError::Disconnected)
        } else if let Ok(_ret) = rx.await {
            Ok(())
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }

    pub async fn session_changed(&mut self, session: &str) -> Result<(), ServiceProxyError> {
        let (tx, rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if self
            .0
            .send(ApplicationRequest::SessionChanged(
                tx,
                String::from(session),
            ))
            .await
            .is_err()
        {
            Err(ServiceProxyError::Disconnected)
        } else if let Ok(_ret) = rx.await {
            Ok(())
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }

    pub async fn application_quit(&mut self) -> Result<(), ServiceProxyError> {
        let (_tx, rx) = tokio::sync::oneshot::channel::<ApplicationResponse>();
        if self.0.send(ApplicationRequest::ServerQuit).await.is_err() {
            Err(ServiceProxyError::Disconnected)
        } else if let Ok(_ret) = rx.await {
            Ok(())
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }
}

pub struct ApplicationServiceWrapper(ApplicationControlProxy, Receiver<ApplicationRequest>);

impl ApplicationServiceWrapper {
    pub fn new_pair(proxy: ApplicationControlProxy) -> (Self, ApplicationServiceProxy) {
        let (sender, rx) = tokio::sync::mpsc::channel::<ApplicationRequest>(1);
        (
            ApplicationServiceWrapper(proxy, rx),
            ApplicationServiceProxy(sender),
        )
    }

    pub async fn serve(&mut self) {
        debug!("Started serving application");
        while let Some(msg) = self.1.recv().await {
            match msg {
                ApplicationRequest::Stop(tx, timeout) => {
                    if let Err(e) = self.stop(timeout).await {
                        let _e = tx.send(ApplicationResponse::Error(e));
                    } else {
                        let _e = tx.send(ApplicationResponse::Ok);
                    }
                }
                ApplicationRequest::Pause(tx, timeout) => {
                    if let Err(e) = self.pause(timeout).await {
                        let _e = tx.send(ApplicationResponse::Error(e));
                    } else {
                        let _e = tx.send(ApplicationResponse::Ok);
                    }
                }
                ApplicationRequest::Event(tx, key, value) => {
                    let timeout = Duration::from_millis(1000);
                    if let Err(e) = self.event((key, value), timeout).await {
                        let _e = tx.send(ApplicationResponse::Error(e));
                    } else {
                        let _e = tx.send(ApplicationResponse::Ok);
                    }
                }
                ApplicationRequest::Property(tx, key, value) => {
                    let timeout = Duration::from_millis(1000);
                    if let Err(e) = self.property_changed((key, value), timeout).await {
                        let _e = tx.send(ApplicationResponse::Error(e));
                    } else {
                        let _e = tx.send(ApplicationResponse::Ok);
                    }
                }
                ApplicationRequest::Resume(tx, _timeout) => {
                    let timeout = Duration::from_millis(1000);
                    if let Err(e) = self.resume(timeout).await {
                        let _e = tx.send(ApplicationResponse::Error(e));
                    } else {
                        let _e = tx.send(ApplicationResponse::Ok);
                    }
                }
                ApplicationRequest::SessionChanged(tx, session) => {
                    let timeout = Duration::from_millis(1000);
                    if let Err(e) = self.session_changed(session, timeout).await {
                        let _e = tx.send(ApplicationResponse::Error(e));
                    } else {
                        let _e = tx.send(ApplicationResponse::Ok);
                    }
                }
                ApplicationRequest::ServerQuit => {
                    break;
                }
            }
        }
        debug!("Application server stopped");
    }

    async fn pause(&mut self, timeout: Duration) -> Result<(), ServiceProxyError> {
        let prop = CallProperties::with_timeout(timeout);
        match self.0.pause(&prop).await {
            Ok(()) => Ok(()),
            Err(e) => Err(ServiceProxyError::Failed),
        } 
    }

    async fn resume(&mut self, timeout: Duration) -> Result<(), ServiceProxyError> {
        let prop = CallProperties::with_timeout(timeout);
        match self.0.resume(&prop).await {
            Ok(()) => Ok(()),
            Err(e) => Err(ServiceProxyError::Failed),
        } 
    }

    async fn session_changed(
        &mut self,
        session: String,
        timeout: Duration,
    ) -> Result<(), ServiceProxyError> {
        
        let prop = CallProperties::with_timeout(timeout);
        
        if let Ok(res) = self
            .0
            .session_changed(session, &prop)
            .await
        {
           Ok(())
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }

    async fn property_changed(
        &mut self,
        property: (String, String),
        timeout: Duration,
    ) -> Result<(), ServiceProxyError> {
        let prop = CallProperties::with_timeout(timeout);
        
        if let Ok(res) = self
            .0
            .on_property_changed(property.0, property.1 ,&prop)
            .await
        {
           Ok(())
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }

    async fn event(
        &mut self,
        property: (String, String),
        timeout: Duration,
    ) -> Result<(), ServiceProxyError> {
        let prop = CallProperties::with_timeout(timeout);
        
        if let Ok(res) = self
            .0
            .handle_event(property.0, property.1 ,&prop)
            .await
        {
           Ok(())
        } else {
            Err(ServiceProxyError::Disconnected)
        }
    }

    async fn stop(&mut self, timeout: Duration) -> Result<(), ServiceProxyError> {
        let prop = CallProperties::with_timeout(timeout);
        match self.0.stop(&prop).await {
            Ok(()) => Ok(()),
            Err(e) => Err(ServiceProxyError::Failed),
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
    fn from(status: application_interface::ReturnStatus) -> Self {
        match status {
            application_interface::ReturnStatus::ERROR => ServiceProxyError::Failed,
            application_interface::ReturnStatus::AGAIN => ServiceProxyError::Retry,
            application_interface::ReturnStatus::OK => panic!("Unexpected as this is not an error"),
        }
    }
}
