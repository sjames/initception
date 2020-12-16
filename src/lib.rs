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

// INITCEPTION client library

extern crate failure;
extern crate failure_derive;
extern crate getopts;
extern crate lazy_static;
extern crate nix;
extern crate tokio;
extern crate tokio_util;
extern crate tracing;
extern crate tracing_subscriber;
extern crate unshare;

pub mod application;
pub mod common;
pub mod context;
pub mod device;
pub mod error;
pub mod init_main;
pub mod initception;
pub mod initrc;
pub mod launcher;
pub mod mount;
pub mod network;
pub mod process;
pub mod server;
pub mod servers;
pub mod sysfs_walker;
mod ueventd;
pub mod userids;
pub mod uventrc_parser;
pub mod zygote;

// re-export ttrpc as all applications need them anyway
pub use ttrpc;
