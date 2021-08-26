

use someip_derive::{*};
use someip::{*};
use thiserror;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use thiserror::Error;
use futures::future::BoxFuture;
use std::sync::Arc;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum  ApplicationManagerError {
    #[error("Unable to Pause")]
    PauseError,
    #[error("Unable to Resume")]
    ResumeError,
    #[error("Unable to Stop")]
    StopError,
}

#[service(name("dev.sabaton.ApplicationControl"))]
#[async_trait]
/// Each application hosts the `ApplicationControl` interface. Applications
/// must fulfil the contracts of this interface or risk being killed for 
/// misbehaviour.
pub trait ApplicationControl {
    /// Pause the application. The application is expected to stop
    /// any major processing activity on reception of this call.
    /// The system may freeze the application after this.
    async fn pause(&self) -> Result<(), ApplicationManagerError>;
    /// Resume normal processing
    async fn resume(&self) -> Result<(), ApplicationManagerError>;
    /// Stop all activities. The process may get killed after this
    /// call is acknowledged.
    async fn stop(&self) -> Result<(), ApplicationManagerError>;
    async fn on_property_changed(&self, key: String, value: String) -> Result<(), ApplicationManagerError>;
    async fn handle_event(&self, key: String, value: String) -> Result<(), ApplicationManagerError>;
}

/*

#[service_impl(ApplicationControl)]
pub struct ApplicationControlImpl {

}

#[async_trait]
impl ApplicationControl for ApplicationControlImpl {
    
    async fn pause(&self) -> Result<(), ApplicationManagerError> {
        Ok(())
    }
    async fn resume(&self) -> Result<(), ApplicationManagerError> {
        Ok(())
    }
    async fn stop(&self) -> Result<(), ApplicationManagerError> {
        Ok(())
    }

    async fn handle_event(&self, key: String, value: String) -> Result<(), ApplicationManagerError> {
        Ok(())
    }

    async fn on_property_changed(&self, key: String, value: String) -> Result<(), ApplicationManagerError> {
        Ok(())
    }
    
}
*/
