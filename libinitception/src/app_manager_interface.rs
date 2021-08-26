

use someip_derive::{*};
use someip::{*};
use thiserror;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use thiserror::Error;
use futures::future::BoxFuture;
use std::sync::Arc;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum  ApplicationControlError {
    #[error("Unable to Pause")]
    PauseError,
    #[error("Unable to Resume")]
    ResumeError,
    #[error("Unable to Stop")]
    StopError,
}

#[interface(name("dev.sabaton.ApplicationControl"))]
#[async_trait]
/// Each application hosts the `ApplicationControl` interface. Applications
/// must fulfil the contracts of this interface or risk being killed for 
/// misbehaviour.
pub trait ApplicationControl {
    /// Pause the application. The application is expected to stop
    /// any major processing activity on reception of this call.
    /// The system may freeze the application after this.
    async fn pause(&self) -> Result<(), ApplicationControlError>;
    /// Resume normal processing
    async fn resume(&self) -> Result<(), ApplicationControlError>;
    /// Stop all activities. The process may get killed after this
    /// call is acknowledged.
    async fn stop(&self) -> Result<(), ApplicationControlError>;
    /// A property that the application is interested in has changed. The 
    /// property key and value is sent.
    async fn on_property_changed(&self, key: String, value: String) -> Result<(), ApplicationControlError>;
    /// An event has been sent to the application.
    async fn handle_event(&self, key: String, value: String) -> Result<(), ApplicationControlError>;
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
#[interface(name("dev.sabaton.LifecycleControl"))]
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


#[derive(Error, Debug, Serialize, Deserialize)]
pub enum  ApplicationManagerError {
    #[error("Unknown Error")]
    Unknown,
    #[error("Property or key not found")]
    NotFound,
    #[error("Property is read-only")]
    ReadOnly,
    #[error("This application has no access to the property")]
    PermissionDenied,
}

#[derive(Serialize, Deserialize)]
pub enum RunningState {
    Running,
    Stopped,
    Paused,
}

#[derive(Serialize, Deserialize)]
pub enum SetPropertyOptions {
    None,
    // if this flag is set and the property is created by this call, it is set read-only
    // The call will fail if the property already exists.
    CreateReadOnly,
}

impl Default for SetPropertyOptions {
    fn default() -> Self {
        SetPropertyOptions::None
    }
}

/// The `ApplicationServer` interface is provided by initception for every application.
/// Applications use this interface to interact with initception
#[interface(name("dev.sabaton.ApplicationServer"))]
#[async_trait]
pub trait ApplicationServer {
    /// Send a heartbeat to the application server. The application also
    /// sends a running counter for the heartbeat
    async fn heartbeat(&self, counter: u32) -> Result<(),ApplicationManagerError>;

    /// The application sends this message to notify the server of state changes
    async fn statechanged(&self, running_state: RunningState) -> Result<(),ApplicationManagerError>;

    /// Retrieve the value of a system property provided the key name
    /// # Returns
    /// The value of the property or `ApplicationManager::NotFound` if the key is not found.
    async fn get_property(&self, key: String) -> Result<String,ApplicationManagerError>;

    /// Set the value of a property. If `SetPropertyOptions::CreateReadOnly` is set,
    /// a new read-only property will be set and marked read-only.
    /// #Returns
    /// If the property already exists and marked read-only, the function will return with `ApplicationManagerError::ReadOnly`
    async fn set_property(&self, key : String, value:String, options: SetPropertyOptions ) -> Result<(),ApplicationManagerError>;

    /// Set a filter for changed properties. Any keys that match the given filter will
    /// be sent to the application via the `ApplicationControl::on_property_changed" method.
    async fn set_property_filter(&self, regex_filter: String) -> Result<(),ApplicationManagerError>;

}
