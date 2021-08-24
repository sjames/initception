

use someip_derive::{*};
use someip::{*};
use thiserror;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum  ApplicationManagerError {
    #[error("Unable to Pause")]
    PauseError,
    #[error("Unable to Resumr")]
    ResumeError,
    #[error("Unable to Stop")]
    StopError,
}

#[service(fields([1]value1:u32,),events([1 ;10]value1:bool))]
#[async_trait]
pub trait ApplicationManager {
    /// Pause the application. The application is expected to stop
    /// any major processing activity on reception of this call.
    /// The system may freeze the application after this.
    async fn pause(&self) -> Result<(), ApplicationManagerError>;
    /// Resume normal processing
    async fn resume(&self) -> Result<(), ApplicationManagerError>;
    /// Stop all activities. The process may get killed after this
    /// call is acknowledged.
    async fn stop(&self) -> Result<(), ApplicationManagerError>;
    async fn notify_property_changed(&self, key: String, value: String) -> Result<(), ApplicationManagerError>;
    async fn notify_event(&self, key: String, value: String) -> Result<(), ApplicationManagerError>;
}

pub struct ApplicationManagerImpl {

}
#[async_trait]
impl ApplicationManager for ApplicationManagerImpl {
    
    async fn pause(&self) -> Result<(), ApplicationManagerError> {
        Ok(())
    }
    async fn resume(&self) -> Result<(), ApplicationManagerError> {
        Ok(())
    }
    async fn stop(&self) -> Result<(), ApplicationManagerError> {
        Ok(())
    }

    async fn notify_event(&self, key: String, value: String) -> Result<(), ApplicationManagerError> {
        Ok(())
    }

    async fn notify_property_changed(&self, key: String, value: String) -> Result<(), ApplicationManagerError> {
        Ok(())
    }

    fn set_value1(&self, _: u32) -> Result<(), FieldError> { Ok(()) }
    
    fn get_value1(&self) -> Result<&u32, FieldError> { 
        Ok(&0)
        }

    
}