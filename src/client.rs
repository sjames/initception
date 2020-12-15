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

// The client library is moved to a different crate to reduce the dependencies introduced by the initception crate
// into the client applications.

use crate::error::InitceptionClientError;

use tokio::net::UnixStream;
use tokio::io::{self, AsyncWriteExt};
use tokio::runtime;
use std::env;
use std::sync::{Arc, RwLock, Mutex};
use std::io::{Error, ErrorKind};


/* 
  Idiomatic callbacks in Rust.
  Read https://stackoverflow.com/questions/41081240/idiomatic-callbacks-in-rust and the answer by user4815162342
  for a fantastic article on supporting callbacks idiomatically.
*/

/// The library implements the protocol between each service and the init process. This protocol runs over the unix domain socket
/// that is created by initception. The socket is an abstract socket. The address of the socket is made available to the service
/// via and environment variable that is set by initception.InitceptionClientError
/// 
/// Protocol
/// The protocol should be as simple as possible to enable even a simple c implementation.InitceptionClientError
/// All messages are line terminated. In the description below, the following annotation indicates the direction of the message
/// (---->)  -> messages sent from initception to the service
/// (<----)  -> messages sent by the service to initception
/// 
/// /// [Lifecycle Messages @LIFECYCLE]
/// 1. @LIFECYCLE=Stopping -> sent when initception is about to stop the service   (---->)
/// 2. @LIFECYCLE=Freezing -> sent when initception is about to freeze the service (---->)
/// 3. @LIFECYCLE=Starting need not be sent as it is implied by the fact that the service is running. (---->)
/// 4. @LIFECYCLE=Started  -> sent by application when it is ready for operation (<----)
/// 5. @LIFECYCLE=Stopped  -> sent by application when it asknowledges the stop  (<----)
/// 6. @LIFECYCLE=Freezed  -> sent by the application when it aknowledges the freeze request (<----)
/// 
/// [Watchdog Messages  @WATCHDOG]
/// 1. @WATCHDOG=Challenge  -> sent by initception      (---->)
/// 2. @WATCHDOG=Response   -> sent by service to initception (<----)
/// 
/// [PropertyMessages  $]
/// The string "PROP" in all the messages below indicate the name of the property and "VALUE" indicates the value of the property
/// It is recommended that you use dot separated properties. Property names cannot contain the following ($@+-~?) or any 
/// space or line end characters.
/// 1. $+PROP=VALUE      -> Property has been added with the given value  (---->)
/// 2. $-PROP            -> Property has been removed (---->)
/// 3. $~PROP=VALUE      -> Property has changed (---->)
/// 4. $+PROP=VALUE      -> Set a property  (<----)
/// 5. $?PROP            -> Get the value of a property. Initception will send the Property value as in message 1.  (<----)

pub enum LifecycleReq {
    Starting,
    Stopping,
    Freezing,
}

#[derive(Debug)]
pub enum LifecycleAck {
    Started,
    Stopped,
    Freezed,
}

pub enum Properties {
    Changed(Option<Vec<(String,String)>>),
    Removed(Option<Vec<(String,String)>>),
    Added(Option<Vec<(String,String)>>),
}

pub struct InitceptionClient<'a> {
    lifecycle_cb: Option<Box<dyn FnMut(LifecycleReq) + 'a>>,
    watchdog_cb : Option<Box<dyn FnMut() +'a>>,
    property_cb : Option<Box<dyn FnMut(Properties) +'a>>,
    writer : Option<tokio::io::WriteHalf<tokio::net::UnixStream>>,
}

//We use the basic scheduler for implementing the synchronous api
lazy_static! {
    static ref  RUNTIME : Arc<Mutex<tokio::runtime::Runtime>> = Arc::new(Mutex::new(runtime::Builder::new().basic_scheduler().build().unwrap()));
}

impl <'a>InitceptionClient<'a> {
    ///Create a new structure
    pub fn new() -> InitceptionClient<'a>  {
        InitceptionClient {
            lifecycle_cb : None,
            watchdog_cb : None,
            property_cb : None,
            writer : None,
        }

    }

    /*
    Builder methods to set callbacks.
    */
    pub fn set_lifecycle_cb<CB : FnMut(LifecycleReq)+'a> (&'a mut self, c: CB) -> &'a mut Self {
        self.lifecycle_cb = Some(Box::new(c));
        self
    }

    pub fn set_watchdog_cb<CB : FnMut()+'a> (&'a mut self, c: CB) -> &'a mut Self {
        self.watchdog_cb = Some(Box::new(c));
        self
    }

    pub fn set_property_cb<CB : FnMut(Properties)+'a> (&'a mut self, c: CB) -> &'a mut Self {
        self.property_cb = Some(Box::new(c));
        self
    }

    pub async fn set_lifecycle_async(celf : Arc<RwLock<Self>>, lifecycle : LifecycleAck) -> Result<(),std::io::Error> {
        println!("Setting lifecycle to {:?}", &lifecycle);
        let mut celf = celf.write().unwrap();
        if let Some(writer) = &mut celf.writer {
            writer.write_all(
                match lifecycle {
                    LifecycleAck::Started => b"@LIFECYCLE=Started\r\n",
                    LifecycleAck::Stopped => b"@LIFECYCLE=Stopped\r\n",
                    LifecycleAck::Freezed => b"@LIFECYCLE=Freezed\r\n",
                }
            ).await
        } else { 
            Err(std::io::Error::new(ErrorKind::Other,"no writer"))
        }
    }

    pub fn set_lifecycle(celf : Arc<RwLock<Self>>, lifecycle : LifecycleAck) -> Result<(),std::io::Error> {
        RUNTIME.lock().unwrap().block_on(Self::set_lifecycle_async(celf,lifecycle))
    }

    pub async fn watchdog_response_async(celf : Arc<RwLock<Self>>) -> Result<(),std::io::Error> {
        let mut celf = celf.write().unwrap();
        if let Some(writer) = &mut celf.writer {
            writer.write_all(b"@WATCHDOG=Response\r\n").await
        } else { 
            Err(std::io::Error::new(ErrorKind::Other,"no writer"))
        }
    }

    pub async fn watchdog_response(celf : Arc<RwLock<Self>>) -> Result<(),std::io::Error> {
        RUNTIME.lock().unwrap().block_on(Self::watchdog_response_async(celf))
    }


    pub async fn set_property_async(celf : Arc<RwLock<Self>>, name: &str, value: &str) -> Result<(),std::io::Error> {
        let mut celf = celf.write().unwrap();
        let cmd = format!("$?{}={}\r\n",name,value);
        if let Some(writer) = &mut celf.writer {
            writer.write_all(cmd.as_bytes()).await
        } else { 
            Err(std::io::Error::new(ErrorKind::Other,"no writer"))
        }
    }

    pub fn set_property(celf : Arc<RwLock<Self>>, name: &str, value: &str) -> Result<(),std::io::Error> {
        RUNTIME.lock().unwrap().block_on(Self::set_property_async(celf,name,value))
    }

    pub async fn get_property_async(celf : Arc<RwLock<Self>>, name: &str) -> Result<(),std::io::Error> {
        let mut celf = celf.write().unwrap();
        let cmd = format!("$?{}\r\n",name);
        if let Some(writer) = &mut celf.writer {
            writer.write_all(cmd.as_bytes()).await
        } else { 
            Err(std::io::Error::new(ErrorKind::Other,"no writer"))
        }
    }

    pub fn get_property(celf : Arc<RwLock<Self>>, name: &str) -> Result<(),std::io::Error> {
        RUNTIME.lock().unwrap().block_on(Self::get_property_async(celf,name))
    }

   
    // This function gobbles up self and returns a Arc::RwLock that you can use to
    // call the static functions of this struct
    pub fn get_arc(self) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(self))
    }

    pub fn start(celf : Arc<RwLock<Self>>) -> Result<(),std::io::Error> {
        RUNTIME.lock().unwrap().block_on(Self::start_async(celf))
    }

    /// Note that this is a future - you need to run it somewhere!
    pub async fn start_async(celf : Arc<RwLock<Self>>) -> Result<(),std::io::Error> {
        if let Ok(uuid) = env::var("NOTIFY_SOCKET") {
            println!("Got NOTIFY SOCKET address as {}",&uuid);
            if let Ok(sock) = UnixStream::connect(String::from("\0")+&uuid).await {
                println!("Connection established");
                let (reader, writer) = tokio::io::split(sock);
                celf.write().unwrap().writer = Some(writer);
                
            } else {
                println!("Connection failed");
            }
            
            Ok(())

        } else {
            panic!("NOTIFY_SOCKET environment variable is not set");
        }

    }

    
}
