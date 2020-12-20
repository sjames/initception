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

use crate::application::src_gen::{application_interface, application_interface_ttrpc as app_int};
/// Application API.  Applications communicate with the init process
/// using ttrpc. These interfaces hide the use of ttrpc.
///
use async_trait::async_trait;

use ttrpc::r#async::{Client, Server};

use std::env;
use std::error::Error;

use std::os::unix::io::FromRawFd;
use std::os::unix::net::UnixStream;
use std::time::SystemTime;

//use std::os::unix::net::UnixStream;

fn get_fd(env_variable_key: &str) -> Result<i32, Box<dyn Error>> {
    if let Ok(address) = env::var(env_variable_key) {
        let raw_fd: i32 = address
            .parse()
            .unwrap_or_else(|_| panic!("Malformed socket number in {}", env_variable_key));
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

pub const NOTIFY_APP_CLIENT_FD: &str = "NOTIFY_APP_CLIENT_FD";
pub const NOTIFY_APP_SERVER_FD: &str = "NOTIFY_APP_SERVER_FD";

// Special names for applications
pub const APPNAME_LIFECYCLE_MANAGER: &str = "lifecycle.manager";

// Application entry

pub struct ApplicationClient {
    manager_proxy: app_int::ApplicationManagerClient,
    lifecycle_proxy : app_int::LifecycleServerClient,
}

impl ApplicationClient {
    pub fn new() -> Self {
        let client_fd = get_client_fd().unwrap();
        let client = Client::new(client_fd);
        ApplicationClient {
            manager_proxy: app_int::ApplicationManagerClient::new(client.clone()),
            lifecycle_proxy: app_int::LifecycleServerClient::new(client),
        }
    }

    /// To be called when the application is running. This function should also be called
    /// on entry to the application to inform the application manager that the app is
    /// running.
    pub async fn set_is_running(&mut self) {
        let mut req = application_interface::StateChangedRequest::new();
        req.set_state(application_interface::StateChangedRequest_State::Running);
        let _reply = self.manager_proxy.statechanged(&req, 0).await;
    }

    pub async fn set_is_paused(&mut self) {
        let mut req = application_interface::StateChangedRequest::new();
        req.set_state(application_interface::StateChangedRequest_State::Paused);
        let _reply = self.manager_proxy.statechanged(&req, 0).await;
    }
    pub async fn set_is_stopped(&mut self) {
        let mut req = application_interface::StateChangedRequest::new();
        req.set_state(application_interface::StateChangedRequest_State::Stopped);
        let _reply = self.manager_proxy.statechanged(&req, 0).await;
    }

    pub async fn send_heartbeat(&mut self) {
        let mut req = application_interface::HeartbeatRequest::new();

        let mut timestamp = application_interface::Timestamp::new();
        let now = SystemTime::now();
        let duration = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        timestamp.set_seconds(duration.as_secs());
        timestamp.set_nanos(duration.subsec_nanos());

        req.set_timestamp(timestamp);
        let _reply = self.manager_proxy.heartbeat(&req, 0).await;
    }

    // The following functions are only available for the lifecycle manager
    pub async fn get_applications(&mut self) -> Option<Vec<String>> {
        let req = application_interface::GetApplicationsRequest::new();
        if let Ok(reply) = self.lifecycle_proxy.get_applications(&req, 0).await {
            Some(reply.name.into())
        } else {
            None
        }
    }

    pub async fn stop_application(&mut self, name:&str) {
        let mut req = application_interface::StopApplicationRequest::new();
        req.set_name(String::from(name));
        if let Ok(_reply) = self.lifecycle_proxy.stop_application(&req, 0).await {
            //Some(reply.get_status().into())
            println!("Stop ok");
        } else {
            //None
            println!("Stop failed");
        }

    }

    pub async fn start_application(&mut self, name:&str) {
        let mut req = application_interface::StartApplicationRequest::new();
        req.set_name(String::from(name));
        if let Ok(_reply) = self.lifecycle_proxy.start_application(&req, 0).await {
            //Some(reply.get_status().into())
            println!("Start ok");
        } else {
            //None
            println!("Start failed");
        }
    }

    pub async fn pause_application(&mut self, name:&str) {
        let mut req = application_interface::PauseApplicationRequest::new();
        req.set_name(String::from(name));
        if let Ok(_reply) = self.lifecycle_proxy.pause_application(&req, 0).await {
            //Some(reply.get_status().into())
            println!("Stop ok");
        } else {
            //None
            println!("Stop failed");
        }

    }
    pub async fn resume_application(&mut self, name:&str) {
        let mut req = application_interface::ResumeApplicationRequest::new();
        req.set_name(String::from(name));
        if let Ok(_reply) = self.lifecycle_proxy.resume_application(&req, 0).await {
            //Some(reply.get_status().into())
            println!("Stop ok");
        } else {
            //None
            println!("Stop failed");
        }

    }
}

struct ApplicationServerInner<P: FnMut(), R: FnMut(), S: FnMut(), C: FnMut(&str)> {
    on_pause: Option<P>,
    on_resume: Option<R>,
    on_stop: Option<S>,
    on_session_changed: Option<C>,
}

pub struct ApplicationServer<P: FnMut(), R: FnMut(), S: FnMut(), C: FnMut(&str)> {
    inner: std::sync::Arc<std::sync::RwLock<ApplicationServerInner<P, R, S, C>>>,
}

#[async_trait]
impl<P, R, S, C> app_int::ApplicationService for ApplicationServer<P, R, S, C>
where
    P: FnMut() + Sync + Send,
    R: FnMut() + Sync + Send,
    S: FnMut() + Sync + Send,
    C: FnMut(&str) + Sync + Send,
{
    async fn pause(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        _req: crate::application::src_gen::application_interface::PauseRequest,
    ) -> ttrpc::Result<crate::application::src_gen::application_interface::PauseResponse> {
        let mut inner = self.inner.write().unwrap();
        if let Some(on_pause) = &mut inner.on_pause {
            on_pause();
            Ok(crate::application::src_gen::application_interface::PauseResponse::default())
        } else {
            Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
                ttrpc::Code::NOT_FOUND,
                "/grpc.ApplicationService/pause is not supported".to_string(),
            )))
        }
    }
    async fn resume(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        _req: crate::application::src_gen::application_interface::ResumeRequest,
    ) -> ttrpc::Result<crate::application::src_gen::application_interface::ResumeResponse> {
        let mut inner = self.inner.write().unwrap();
        if let Some(on_resume) = &mut inner.on_resume {
            on_resume();
            Ok(crate::application::src_gen::application_interface::ResumeResponse::default())
        } else {
            Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
                ttrpc::Code::NOT_FOUND,
                "/grpc.ApplicationService/resume is not supported".to_string(),
            )))
        }
    }
    async fn stop(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        _req: crate::application::src_gen::application_interface::StopRequest,
    ) -> ttrpc::Result<crate::application::src_gen::application_interface::StopResponse> {
        let mut inner = self.inner.write().unwrap();
        if let Some(on_stop) = &mut inner.on_stop {
            on_stop();
            Ok(crate::application::src_gen::application_interface::StopResponse::default())
        } else {
            Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
                ttrpc::Code::NOT_FOUND,
                "/grpc.ApplicationService/stop is not supported".to_string(),
            )))
        }
    }
    async fn session_changed(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        req: crate::application::src_gen::application_interface::SessionChangedRequest,
    ) -> ttrpc::Result<crate::application::src_gen::application_interface::SessionChangedResponse>
    {
        let mut inner = self.inner.write().unwrap();
        if let Some(on_session_changed) = &mut inner.on_session_changed {
            on_session_changed(req.session_name.as_str());
            Ok(
                crate::application::src_gen::application_interface::SessionChangedResponse::default(
                ),
            )
        } else {
            Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
                ttrpc::Code::NOT_FOUND,
                "/grpc.ApplicationService/session_changed is not supported".to_string(),
            )))
        }
    }
}

impl<'a, P, R, S, C> ApplicationServer<P, R, S, C>
where
    P: FnMut() + Send + Sync + 'a,
    R: FnMut() + Send + Sync + 'a,
    S: FnMut() + Send + Sync + 'a,
    C: FnMut(&str) + Send + Sync + 'a,
{
    pub fn new(on_pause: P, on_resume: R, on_stop: S, on_session_changed: C) -> Self {
        ApplicationServer {
            inner: {
                std::sync::Arc::new(std::sync::RwLock::new(ApplicationServerInner {
                    on_pause: Some(on_pause),
                    on_resume: Some(on_resume),
                    on_stop: Some(on_stop),
                    on_session_changed: Some(on_session_changed),
                }))
            },
        }
    }

    pub fn get_server<'b>(on_pause: P, on_resume: R, on_stop: S, on_session_changed: C) -> Server
    where
        P: FnMut() + Send + Sync + 'static,
        R: FnMut() + Send + Sync + 'static,
        S: FnMut() + Send + Sync + 'static,
        C: FnMut(&str) + Send + Sync + 'static,
    {
        let service = Box::new(ApplicationServer::new(
            on_pause,
            on_resume,
            on_stop,
            on_session_changed,
        ))
            as Box<
                dyn crate::application::src_gen::application_interface_ttrpc::ApplicationService
                    + Send
                    + Sync,
            >;
        let service = std::sync::Arc::new(service);
        let service =
            crate::application::src_gen::application_interface_ttrpc::create_application_service(
                service,
            );
        Server::new().register_service(service).set_domain_unix()
    }
}

pub async fn start_server(mut server: Server) -> Server {
    let server_fd = get_server_fd().unwrap();
    let stream = unsafe { UnixStream::from_raw_fd(server_fd) };
    if let Err(e) = server.start_single(stream).await.map_err(Box::new) {
        panic!("Cannot start server due to: {}", e);
    }
    server
}
