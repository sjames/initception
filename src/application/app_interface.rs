use essrpc::essrpc;
use essrpc::transports::{
    BincodeAsyncClientTransport,
    ReadWrite,
};
use essrpc::{RPCError};

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;
use std::result::Result;

#[essrpc(async)]
pub trait ApplicationInterface {
    fn pause(&mut self) -> Result<(),ApplicationError>;
    fn resume(&mut self) -> Result<(),ApplicationError>;
    fn stop(&mut self) -> Result<(),ApplicationError>;
    fn session_changed(&mut self,session: String) -> Result<(),ApplicationError>;
}


#[essrpc(async)]
pub trait ApplicationManagerInterface {
    fn heartbeat(&mut self) -> Result<(),ApplicationError>;
    fn started(&mut self) -> Result<(),ApplicationError>;
    fn stopped(&mut self) -> Result<(),ApplicationError>;
    fn paused(&mut self) -> Result<(),ApplicationError>;
    fn running(&mut self) -> Result<(),ApplicationError>;
}

/// Application errors
#[derive(Debug, Deserialize, Serialize)]
pub struct ApplicationError {
    msg: String,
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {}", self.msg)
    }
}

impl std::error::Error for ApplicationError {}
impl From<essrpc::RPCError> for ApplicationError {
    fn from(error: essrpc::RPCError) -> Self {
        ApplicationError {
            msg: format!("{}", error),
        }
    }
}