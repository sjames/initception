/// This is where all the common types go
use crate::context::ServiceIndex;

pub enum TaskMessage {
    RequestLaunch(ServiceIndex),
    ProcessLaunched(ServiceIndex),
    ProcessRunning(ServiceIndex),
    ProcessPaused(ServiceIndex),  // process has confirmed the pause
    ProcessStopped(ServiceIndex), // process has confirmed the stop
    ProcessExited(ServiceIndex),
    ConfigureNetworkLoopback,        // configure and enable the lo interface
    UeventReady,                     // The Uevent task is ready to listen for events
    DeviceChanged(DeviceChangeInfo), // a device has been added
    UnitSuccess(ServiceIndex),       // A unit was successfully executed
}

pub enum DeviceChangeInfo {
    Added(String),
    Removed(String),
    Changed(String),
}

pub type TxHandle = tokio::sync::mpsc::Sender<TaskMessage>;
pub type SyncTxHandle = std::sync::mpsc::Sender<TaskMessage>;
