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

/// This is where all the common types go
use crate::context::ServiceIndex;
use std::sync::mpsc::Sender;

pub enum TaskReply {
    Ok,
    Error,
}

impl TaskReply {
    pub fn is_ok(&self) -> bool {
        matches!(self, TaskReply::Ok)
    }
}

impl Default for TaskReply {
    fn default() -> Self {
        TaskReply::Ok
    }
}
pub enum TaskMessage {
    RequestLaunch(ServiceIndex,Option<Sender<TaskReply>>),
    RequestStop(ServiceIndex,Option<Sender<TaskReply>>),
    ProcessLaunched(ServiceIndex,Option<Sender<TaskReply>>),
    ProcessRunning(ServiceIndex,Option<Sender<TaskReply>>),
    ProcessPaused(ServiceIndex,Option<Sender<TaskReply>>),  // process has confirmed the pause
    ProcessStopped(ServiceIndex,Option<Sender<TaskReply>>), // process has confirmed the stop
    ProcessExited(ServiceIndex,Option<Sender<TaskReply>>),
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
