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

use std::env;
use std::error::Error;
use std::os::unix::io::FromRawFd;
//use std::os::unix::net::UnixStream;

fn get_fd(env_variable_key: &str) -> Result<i32, Box<dyn Error>> {
    if let Ok(address) = env::var(env_variable_key) {
        let raw_fd: i32 = address
            .parse()
            .expect(&format!("Malformed socket number in {}", env_variable_key));
        //let std_stream = unsafe { UnixStream::from_raw_fd(raw_fd) };
        Ok(raw_fd)
    } else {
        Err(Box::new(std::io::Error::from(
            std::io::ErrorKind::AddrNotAvailable,
        )))
    }
}

/// Get the UnixStream for the client on the application
pub fn get_client_fd() -> Result<i32, Box<dyn Error>> {
    get_fd(NOTIFY_APP_CLIENT_FD)
}

/// Get the UnixStream for the server on the application
pub fn get_server_fd() -> Result<i32, Box<dyn Error>> {
    get_fd(NOTIFY_APP_SERVER_FD)
}

pub const NOTIFY_APP_CLIENT_FD:&str = "NOTIFY_APP_CLIENT_FD";
pub const NOTIFY_APP_SERVER_FD:&str = "NOTIFY_APP_SERVER_FD";

