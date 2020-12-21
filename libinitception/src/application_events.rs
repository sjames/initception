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

// Application Event identifiers

// An application has stopped. Value will contain the application name
pub const EVENT_APPLICATION_STOPPED: &str = "int.lifecycle.event.application.stopped";

// An application has started. Value will contain the application name
pub const EVENT_APPLICATION_STARTED: &str = "int.lifecycle.event.application.started";

// An application has missed a heartbeat
pub const EVENT_APPLICATION_HEARTBEAT_MISSED: &str =
    "int.lifecycle.event.application.watchdog.missed";

// An application has crashed
pub const EVENT_APPLICATION_CRASHED: &str = "int.lifecycle.event.application.crashed";
