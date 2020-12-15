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
pub mod sysfs_walker;
mod ueventd;
pub mod userids;
pub mod uventrc_parser;
pub mod zygote;

// re-export ttrpc as all applications need them anyway
pub use ttrpc;