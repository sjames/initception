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

//use crate::src_gen::{application_interface, application_interface_ttrpc as app_int};
/// Application API.  Applications communicate with the init process
/// using ttrpc. These interfaces hide the use of ttrpc.
///
use async_trait::async_trait;

//pub use ttrpc::r#async::{Client, Server};

use std::env;
use std::error::Error;

use std::os::unix::io::FromRawFd;
use std::os::unix::net::UnixStream;
use std::time::SystemTime;

use thiserror::Error;
// Error returned by Applications

use crate::app_manager_interface::*;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Application is not ready")]
    NotReady,
    #[error("Application failed to change state")]
    Failed,
    #[error("unknown Application error")]
    Unknown,
}

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

/*

// Application entry
pub struct ApplicationClient {
    manager_proxy: ApplicationServerProxy,
    lifecycle_proxy: LifecycleControlProxy,
}

impl ApplicationClient {
    pub fn new() -> Self {
        let client_fd = get_client_fd().unwrap();
        //let client = Client::new(client_fd);
        let config = Configuration::default();
        let manager_proxy = ApplicationServerProxy::new(id_of_service(ApplicationServerProxy::service_name()).unwrap(),0, config.clone());
        let client_dispatcher = manager_proxy.get_dispatcher();
        let lifecycle_proxy = LifecycleControlProxy::new(id_of_service(LifecycleControlProxy::service_name()).unwrap(),config.clone(),client_dispatcher);

        ApplicationClient {
            manager_proxy,
            lifecycle_proxy,
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

    pub async fn get_property(&mut self, key: String) -> Option<String> {
        let mut req = application_interface::GetPropertyRequest::new();

        req.set_key(key);
        if let Ok(mut reply) = self.manager_proxy.get_property(&req, 0).await {
            Some(reply.take_value())
        } else {
            None
        }
    }

    pub async fn set_property(
        &mut self,
        key: String,
        value: String,
    ) -> Result<(), application_interface::ReturnStatus> {
        let mut req = application_interface::SetPropertyRequest::new();
        req.set_key(key);
        req.set_value(value);
        if let Ok(reply) = self.manager_proxy.set_property(&req, 0).await {
            match reply.get_status() {
                application_interface::ReturnStatus::OK => Ok(()),
                _ => Err(reply.get_status()),
            }
        } else {
            Err(application_interface::ReturnStatus::ERROR)
        }
    }

    pub async fn add_property_filter(
        &mut self,
        key_match: String,
    ) -> Result<(), application_interface::ReturnStatus> {
        let mut req = application_interface::AddPropertyFilterRequest::new();
        req.set_regex(key_match);

        if let Ok(reply) = self.manager_proxy.add_property_filter(&req, 0).await {
            match reply.get_status() {
                application_interface::ReturnStatus::OK => Ok(()),
                _ => Err(reply.get_status()),
            }
        } else {
            Err(application_interface::ReturnStatus::ERROR)
        }
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

    pub async fn stop_application(&mut self, name: &str) {
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

    pub async fn start_application(&mut self, name: &str) {
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

    pub async fn pause_application(&mut self, name: &str) {
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
    pub async fn resume_application(&mut self, name: &str) {
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

struct ApplicationServerInner<
    P: FnMut() -> Result<(), AppError>,
    R: FnMut() -> Result<(), AppError>,
    S: FnMut() -> Result<(), AppError>,
    C: FnMut(&str) -> Result<(), AppError>,
    T: FnMut(String, String) -> Result<(), AppError>,
    E: FnMut(String, String) -> Result<(), AppError>,
> {
    on_pause: Option<P>,
    on_resume: Option<R>,
    on_stop: Option<S>,
    on_session_changed: Option<C>,
    on_property_changed: Option<T>,
    on_event: Option<E>,
}

pub struct ApplicationServer<
    P: FnMut() -> Result<(), AppError>,
    R: FnMut() -> Result<(), AppError>,
    S: FnMut() -> Result<(), AppError>,
    C: FnMut(&str) -> Result<(), AppError>,
    T: FnMut(String, String) -> Result<(), AppError>,
    E: FnMut(String, String) -> Result<(), AppError>,
> {
    inner: std::sync::Arc<std::sync::RwLock<ApplicationServerInner<P, R, S, C, T, E>>>,
}

#[async_trait]
impl<P, R, S, C, T, E> app_int::ApplicationService for ApplicationServer<P, R, S, C, T, E>
where
    P: FnMut() -> Result<(), AppError> + Sync + Send,
    R: FnMut() -> Result<(), AppError> + Sync + Send,
    S: FnMut() -> Result<(), AppError> + Sync + Send,
    C: FnMut(&str) -> Result<(), AppError> + Sync + Send,
    T: FnMut(String, String) -> Result<(), AppError> + Sync + Send,
    E: FnMut(String, String) -> Result<(), AppError> + Sync + Send,
{
    async fn pause(
        &self,
        _ctx: &ttrpc::r#async::TtrpcContext,
        _req: crate::src_gen::application_interface::PauseRequest,
    ) -> ttrpc::Result<crate::src_gen::application_interface::PauseResponse> {
        let mut inner = self.inner.write().unwrap();
        if let Some(on_pause) = &mut inner.on_pause {
            let mut response = crate::src_gen::application_interface::PauseResponse::default();
            match on_pause() {
                Ok(_) => {}
                Err(_) => response.set_status(application_interface::ReturnStatus::ERROR),
            }
            Ok(response)
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
        _req: crate::src_gen::application_interface::ResumeRequest,
    ) -> ttrpc::Result<crate::src_gen::application_interface::ResumeResponse> {
        let mut inner = self.inner.write().unwrap();
        if let Some(on_resume) = &mut inner.on_resume {
            let mut response = crate::src_gen::application_interface::ResumeResponse::default();
            match on_resume() {
                Ok(_) => {}
                Err(_) => response.set_status(application_interface::ReturnStatus::ERROR),
            }
            Ok(response)
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
        _req: crate::src_gen::application_interface::StopRequest,
    ) -> ttrpc::Result<crate::src_gen::application_interface::StopResponse> {
        let mut inner = self.inner.write().unwrap();
        if let Some(on_stop) = &mut inner.on_stop {
            let mut response = crate::src_gen::application_interface::StopResponse::default();
            match on_stop() {
                Ok(_) => {}
                Err(_) => response.set_status(application_interface::ReturnStatus::ERROR),
            }
            Ok(response)
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
        req: crate::src_gen::application_interface::SessionChangedRequest,
    ) -> ttrpc::Result<crate::src_gen::application_interface::SessionChangedResponse> {
        let mut inner = self.inner.write().unwrap();
        if let Some(on_session_changed) = &mut inner.on_session_changed {
            let mut response =
                crate::src_gen::application_interface::SessionChangedResponse::default();
            match on_session_changed(req.session_name.as_str()) {
                Ok(_) => {}
                Err(_) => response.set_status(application_interface::ReturnStatus::ERROR),
            }
            Ok(response)
        } else {
            Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
                ttrpc::Code::NOT_FOUND,
                "/grpc.ApplicationService/session_changed is not supported".to_string(),
            )))
        }
    }

    async fn property_changed(
        &self,
        _ctx: &::ttrpc::r#async::TtrpcContext,
        mut req: crate::src_gen::application_interface::PropertyChangedRequest,
    ) -> ::ttrpc::Result<crate::src_gen::application_interface::PropertyChangedResponse> {
        let mut inner = self.inner.write().unwrap();
        if let Some(on_property_changed) = &mut inner.on_property_changed {
            let mut response =
                crate::src_gen::application_interface::PropertyChangedResponse::default();
            match on_property_changed(req.take_key(), req.take_value()) {
                Ok(_) => {}
                Err(_) => {} // Don't care about errors for property
            }
            Ok(response)
        } else {
            Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
                ttrpc::Code::NOT_FOUND,
                "/grpc.ApplicationService/property_changed is not supported".to_string(),
            )))
        }
    }

    async fn event(
        &self,
        _ctx: &::ttrpc::r#async::TtrpcContext,
        mut req: super::application_interface::EventRequest,
    ) -> ::ttrpc::Result<super::application_interface::EventResponse> {
        let mut inner = self.inner.write().unwrap();
        if let Some(on_event) = &mut inner.on_event {
            let response = crate::src_gen::application_interface::EventResponse::default();
            match on_event(req.take_key(), req.take_value()) {
                Ok(_) => {}
                Err(_) => {} // Don't care about errors for property
            }
            Ok(response)
        } else {
            Err(ttrpc::Error::RpcStatus(ttrpc::get_status(
                ttrpc::Code::NOT_FOUND,
                "/grpc.ApplicationService/event is not supported".to_string(),
            )))
        }
    }
}

impl<'a, P, R, S, C, T, E> ApplicationServer<P, R, S, C, T, E>
where
    P: FnMut() -> Result<(), AppError> + Send + Sync + 'a,
    R: FnMut() -> Result<(), AppError> + Send + Sync + 'a,
    S: FnMut() -> Result<(), AppError> + Send + Sync + 'a,
    C: FnMut(&str) -> Result<(), AppError> + Send + Sync + 'a,
    T: FnMut(String, String) -> Result<(), AppError> + Send + Sync + 'a,
    E: FnMut(String, String) -> Result<(), AppError> + Send + Sync + 'a,
{
    pub fn new(
        on_pause: P,
        on_resume: R,
        on_stop: S,
        on_session_changed: C,
        on_property_changed: T,
        on_event: E,
    ) -> Self {
        ApplicationServer {
            inner: {
                std::sync::Arc::new(std::sync::RwLock::new(ApplicationServerInner {
                    on_pause: Some(on_pause),
                    on_resume: Some(on_resume),
                    on_stop: Some(on_stop),
                    on_session_changed: Some(on_session_changed),
                    on_property_changed: Some(on_property_changed),
                    on_event: Some(on_event),
                }))
            },
        }
    }

    pub fn get_server<'b>(
        on_pause: P,
        on_resume: R,
        on_stop: S,
        on_session_changed: C,
        on_property_changed: T,
        on_event: E,
    ) -> Server
    where
        P: FnMut() -> Result<(), AppError> + Send + Sync + 'static,
        R: FnMut() -> Result<(), AppError> + Send + Sync + 'static,
        S: FnMut() -> Result<(), AppError> + Send + Sync + 'static,
        C: FnMut(&str) -> Result<(), AppError> + Send + Sync + 'static,
        T: FnMut(String, String) -> Result<(), AppError> + Send + Sync + 'static,
        E: FnMut(String, String) -> Result<(), AppError> + Send + Sync + 'static,
    {
        let service = Box::new(ApplicationServer::new(
            on_pause,
            on_resume,
            on_stop,
            on_session_changed,
            on_property_changed,
            on_event,
        ))
            as Box<
                dyn crate::src_gen::application_interface_ttrpc::ApplicationService + Send + Sync,
            >;
        let service = std::sync::Arc::new(service);
        let service =
            crate::src_gen::application_interface_ttrpc::create_application_service(service);
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

*/