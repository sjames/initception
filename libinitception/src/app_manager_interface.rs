

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
    /// A property that the application is interested in has changed. The 
    /// property key and value is sent.
    async fn on_property_changed(&self, key: String, value: String) -> Result<(), ApplicationManagerError>;
    /// An event has been sent to the application.
    async fn handle_event(&self, key: String, value: String) -> Result<(), ApplicationManagerError>;
}


#[derive(Error, Debug, Serialize, Deserialize)]
pub enum  LifecycleControlError {
    #[error("Unknown Error")]
    Unknown,
}

#[derive(Serialize, Deserialize)]
pub enum ApplicationStatus {
    Unknown,
    Stopped,
    Running,
    Paused,
}

#[derive(Serialize, Deserialize)]
pub enum ShutdownType {
    /// Normal shutdown
    Normal,
    /// Fast shutdown
    Fast,
    /// Super fast shutdown! This is an emergency
    Emergency,
}

/// The LifecycleControl interface is provided by initception. The Lifecycle manager
/// application uses this interface to manage the lifecycle of the entire system.
/// The lifecycle manager is a privileged application that gets access to this
/// interface at startup via an anonymous UnixDomain socket.
#[service(name("dev.sabaton.LifecycleControl"))]
#[async_trait]
pub trait LifecycleControl {
    /// Return the list of all applications in the system.
    async fn get_applications(&self) -> Result<Vec<String>,LifecycleControlError>;
    /// Return the status of an application. An application is identified by name.
    async fn get_application_status(&self,name: String) -> Result<ApplicationStatus,LifecycleControlError>;
    /// Start the application. 
    /// # Returns
    /// On success, the function returns the time it took to start the application
    async fn start_application(&self,name: String) -> Result<std::time::Duration,LifecycleControlError>;
    /// Restart the application. This will force a shutdown and then an restart.
    /// # Returns
    /// On success, the function returns the time it took to restart the application
    async fn restart_application(&self,name: String) -> Result<std::time::Duration,LifecycleControlError>;

    /// Pause the application. 
    /// # Returns
    /// On success, the function returns the time it took to pause the application
    async fn pause_application(&self,name: String) -> Result<std::time::Duration,LifecycleControlError>;

    /// Resume the application. 
    /// # Returns
    /// On success, the function returns the time it took to resume the application
    async fn resume_application(&self,name: String) -> Result<std::time::Duration,LifecycleControlError>;

    /// Stop the application. 
    /// # Returns
    /// On success, the function returns the time it took to stop the application
    async fn stop_application(&self,name: String) -> Result<std::time::Duration,LifecycleControlError>;

    /// Prepare the system for freeze. 
    /// # Returns
    /// On success, the system is ready to go to sleep
    async fn prepare_system_freeze(&self, shutdown_type: ShutdownType) -> Result<(),LifecycleControlError>;

    /// Prepare the system for shutdown. 
    /// # Returns
    /// On success, the system is ready to be shutdown
    async fn prepare_system_shutdown(&self, shutdown_type: ShutdownType) -> Result<(),LifecycleControlError>;

    /// Shutdown the system
    /// # Returns
    /// TBD
    async fn shutdown_system(&self, shutdown_type: ShutdownType) -> Result<(),LifecycleControlError>;

}


