// This file is generated by ttrpc-compiler 0.3.2. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clipto_camel_casepy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]
use protobuf::{CodedInputStream, CodedOutputStream, Message};
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Clone)]
pub struct ApplicationServiceClient {
    client: ::ttrpc::r#async::Client,
}

impl ApplicationServiceClient {
    pub fn new(client: ::ttrpc::r#async::Client) -> Self {
        ApplicationServiceClient {
            client: client,
        }
    }

    pub async fn pause(&mut self, req: &super::application_interface::PauseRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::PauseResponse> {
        let mut cres = super::application_interface::PauseResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.ApplicationService", "pause", cres);
    }

    pub async fn resume(&mut self, req: &super::application_interface::ResumeRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::ResumeResponse> {
        let mut cres = super::application_interface::ResumeResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.ApplicationService", "resume", cres);
    }

    pub async fn stop(&mut self, req: &super::application_interface::StopRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::StopResponse> {
        let mut cres = super::application_interface::StopResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.ApplicationService", "stop", cres);
    }

    pub async fn session_changed(&mut self, req: &super::application_interface::SessionChangedRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::SessionChangedResponse> {
        let mut cres = super::application_interface::SessionChangedResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.ApplicationService", "session_changed", cres);
    }

    pub async fn property_changed(&mut self, req: &super::application_interface::PropertyChangedRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::PropertyChangedResponse> {
        let mut cres = super::application_interface::PropertyChangedResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.ApplicationService", "property_changed", cres);
    }

    pub async fn event(&mut self, req: &super::application_interface::EventRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::EventResponse> {
        let mut cres = super::application_interface::EventResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.ApplicationService", "event", cres);
    }
}

struct PauseMethod {
    service: Arc<std::boxed::Box<dyn ApplicationService + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for PauseMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, PauseRequest, pause);
    }
}

struct ResumeMethod {
    service: Arc<std::boxed::Box<dyn ApplicationService + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for ResumeMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, ResumeRequest, resume);
    }
}

struct StopMethod {
    service: Arc<std::boxed::Box<dyn ApplicationService + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for StopMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, StopRequest, stop);
    }
}

struct SessionChangedMethod {
    service: Arc<std::boxed::Box<dyn ApplicationService + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for SessionChangedMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, SessionChangedRequest, session_changed);
    }
}

struct PropertyChangedMethod {
    service: Arc<std::boxed::Box<dyn ApplicationService + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for PropertyChangedMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, PropertyChangedRequest, property_changed);
    }
}

struct EventMethod {
    service: Arc<std::boxed::Box<dyn ApplicationService + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for EventMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, EventRequest, event);
    }
}

#[async_trait]
pub trait ApplicationService: Sync {
    async fn pause(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::PauseRequest) -> ::ttrpc::Result<super::application_interface::PauseResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationService/pause is not supported".to_string())))
    }
    async fn resume(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::ResumeRequest) -> ::ttrpc::Result<super::application_interface::ResumeResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationService/resume is not supported".to_string())))
    }
    async fn stop(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::StopRequest) -> ::ttrpc::Result<super::application_interface::StopResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationService/stop is not supported".to_string())))
    }
    async fn session_changed(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::SessionChangedRequest) -> ::ttrpc::Result<super::application_interface::SessionChangedResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationService/session_changed is not supported".to_string())))
    }
    async fn property_changed(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::PropertyChangedRequest) -> ::ttrpc::Result<super::application_interface::PropertyChangedResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationService/property_changed is not supported".to_string())))
    }
    async fn event(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::EventRequest) -> ::ttrpc::Result<super::application_interface::EventResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationService/event is not supported".to_string())))
    }
}

pub fn create_application_service(service: Arc<std::boxed::Box<dyn ApplicationService + Send + Sync>>) -> HashMap <String, Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>> {
    let mut methods = HashMap::new();

    methods.insert("/grpc.ApplicationService/pause".to_string(),
                    std::boxed::Box::new(PauseMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.ApplicationService/resume".to_string(),
                    std::boxed::Box::new(ResumeMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.ApplicationService/stop".to_string(),
                    std::boxed::Box::new(StopMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.ApplicationService/session_changed".to_string(),
                    std::boxed::Box::new(SessionChangedMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.ApplicationService/property_changed".to_string(),
                    std::boxed::Box::new(PropertyChangedMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.ApplicationService/event".to_string(),
                    std::boxed::Box::new(EventMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods
}

#[derive(Clone)]
pub struct ApplicationManagerClient {
    client: ::ttrpc::r#async::Client,
}

impl ApplicationManagerClient {
    pub fn new(client: ::ttrpc::r#async::Client) -> Self {
        ApplicationManagerClient {
            client: client,
        }
    }

    pub async fn heartbeat(&mut self, req: &super::application_interface::HeartbeatRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::HeartbeatResponse> {
        let mut cres = super::application_interface::HeartbeatResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.ApplicationManager", "heartbeat", cres);
    }

    pub async fn statechanged(&mut self, req: &super::application_interface::StateChangedRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::StateChangedResponse> {
        let mut cres = super::application_interface::StateChangedResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.ApplicationManager", "statechanged", cres);
    }

    pub async fn get_property(&mut self, req: &super::application_interface::GetPropertyRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::GetPropertyResponse> {
        let mut cres = super::application_interface::GetPropertyResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.ApplicationManager", "get_property", cres);
    }

    pub async fn set_property(&mut self, req: &super::application_interface::SetPropertyRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::SetPropertyResponse> {
        let mut cres = super::application_interface::SetPropertyResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.ApplicationManager", "set_property", cres);
    }

    pub async fn add_property_filter(&mut self, req: &super::application_interface::AddPropertyFilterRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::AddPropertyFilterResponse> {
        let mut cres = super::application_interface::AddPropertyFilterResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.ApplicationManager", "add_property_filter", cres);
    }
}

struct HeartbeatMethod {
    service: Arc<std::boxed::Box<dyn ApplicationManager + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for HeartbeatMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, HeartbeatRequest, heartbeat);
    }
}

struct StatechangedMethod {
    service: Arc<std::boxed::Box<dyn ApplicationManager + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for StatechangedMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, StateChangedRequest, statechanged);
    }
}

struct GetPropertyMethod {
    service: Arc<std::boxed::Box<dyn ApplicationManager + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for GetPropertyMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, GetPropertyRequest, get_property);
    }
}

struct SetPropertyMethod {
    service: Arc<std::boxed::Box<dyn ApplicationManager + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for SetPropertyMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, SetPropertyRequest, set_property);
    }
}

struct AddPropertyFilterMethod {
    service: Arc<std::boxed::Box<dyn ApplicationManager + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for AddPropertyFilterMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, AddPropertyFilterRequest, add_property_filter);
    }
}

#[async_trait]
pub trait ApplicationManager: Sync {
    async fn heartbeat(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::HeartbeatRequest) -> ::ttrpc::Result<super::application_interface::HeartbeatResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationManager/heartbeat is not supported".to_string())))
    }
    async fn statechanged(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::StateChangedRequest) -> ::ttrpc::Result<super::application_interface::StateChangedResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationManager/statechanged is not supported".to_string())))
    }
    async fn get_property(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::GetPropertyRequest) -> ::ttrpc::Result<super::application_interface::GetPropertyResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationManager/get_property is not supported".to_string())))
    }
    async fn set_property(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::SetPropertyRequest) -> ::ttrpc::Result<super::application_interface::SetPropertyResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationManager/set_property is not supported".to_string())))
    }
    async fn add_property_filter(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::AddPropertyFilterRequest) -> ::ttrpc::Result<super::application_interface::AddPropertyFilterResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationManager/add_property_filter is not supported".to_string())))
    }
}

pub fn create_application_manager(service: Arc<std::boxed::Box<dyn ApplicationManager + Send + Sync>>) -> HashMap <String, Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>> {
    let mut methods = HashMap::new();

    methods.insert("/grpc.ApplicationManager/heartbeat".to_string(),
                    std::boxed::Box::new(HeartbeatMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.ApplicationManager/statechanged".to_string(),
                    std::boxed::Box::new(StatechangedMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.ApplicationManager/get_property".to_string(),
                    std::boxed::Box::new(GetPropertyMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.ApplicationManager/set_property".to_string(),
                    std::boxed::Box::new(SetPropertyMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.ApplicationManager/add_property_filter".to_string(),
                    std::boxed::Box::new(AddPropertyFilterMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods
}

#[derive(Clone)]
pub struct LifecycleServerClient {
    client: ::ttrpc::r#async::Client,
}

impl LifecycleServerClient {
    pub fn new(client: ::ttrpc::r#async::Client) -> Self {
        LifecycleServerClient {
            client: client,
        }
    }

    pub async fn get_applications(&mut self, req: &super::application_interface::GetApplicationsRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::GetApplicationsResponse> {
        let mut cres = super::application_interface::GetApplicationsResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.LifecycleServer", "get_applications", cres);
    }

    pub async fn get_application_status(&mut self, req: &super::application_interface::GetApplicationStatusRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::GetApplicationStatusResponse> {
        let mut cres = super::application_interface::GetApplicationStatusResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.LifecycleServer", "get_application_status", cres);
    }

    pub async fn start_application(&mut self, req: &super::application_interface::StartApplicationRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::StartApplicationResponse> {
        let mut cres = super::application_interface::StartApplicationResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.LifecycleServer", "start_application", cres);
    }

    pub async fn restart_application(&mut self, req: &super::application_interface::RestartApplicationRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::RestartApplicationResponse> {
        let mut cres = super::application_interface::RestartApplicationResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.LifecycleServer", "restart_application", cres);
    }

    pub async fn pause_application(&mut self, req: &super::application_interface::PauseApplicationRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::PauseApplicationResponse> {
        let mut cres = super::application_interface::PauseApplicationResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.LifecycleServer", "pause_application", cres);
    }

    pub async fn resume_application(&mut self, req: &super::application_interface::ResumeApplicationRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::ResumeApplicationResponse> {
        let mut cres = super::application_interface::ResumeApplicationResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.LifecycleServer", "resume_application", cres);
    }

    pub async fn stop_application(&mut self, req: &super::application_interface::StopApplicationRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::StopApplicationResponse> {
        let mut cres = super::application_interface::StopApplicationResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.LifecycleServer", "stop_application", cres);
    }

    pub async fn prepare_system_freeze(&mut self, req: &super::application_interface::PrepareFreezeRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::PrepareFreezeResponse> {
        let mut cres = super::application_interface::PrepareFreezeResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.LifecycleServer", "prepare_system_freeze", cres);
    }

    pub async fn prepare_system_shutdown(&mut self, req: &super::application_interface::PrepareShutdownRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::PrepareShutdownResponse> {
        let mut cres = super::application_interface::PrepareShutdownResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.LifecycleServer", "prepare_system_shutdown", cres);
    }

    pub async fn shutdown_system(&mut self, req: &super::application_interface::ShutdownRequest, timeout_nano: i64) -> ::ttrpc::Result<super::application_interface::ShutdownResponse> {
        let mut cres = super::application_interface::ShutdownResponse::new();
        ::ttrpc::async_client_request!(self, req, timeout_nano, "grpc.LifecycleServer", "shutdown_system", cres);
    }
}

struct GetApplicationsMethod {
    service: Arc<std::boxed::Box<dyn LifecycleServer + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for GetApplicationsMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, GetApplicationsRequest, get_applications);
    }
}

struct GetApplicationStatusMethod {
    service: Arc<std::boxed::Box<dyn LifecycleServer + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for GetApplicationStatusMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, GetApplicationStatusRequest, get_application_status);
    }
}

struct StartApplicationMethod {
    service: Arc<std::boxed::Box<dyn LifecycleServer + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for StartApplicationMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, StartApplicationRequest, start_application);
    }
}

struct RestartApplicationMethod {
    service: Arc<std::boxed::Box<dyn LifecycleServer + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for RestartApplicationMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, RestartApplicationRequest, restart_application);
    }
}

struct PauseApplicationMethod {
    service: Arc<std::boxed::Box<dyn LifecycleServer + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for PauseApplicationMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, PauseApplicationRequest, pause_application);
    }
}

struct ResumeApplicationMethod {
    service: Arc<std::boxed::Box<dyn LifecycleServer + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for ResumeApplicationMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, ResumeApplicationRequest, resume_application);
    }
}

struct StopApplicationMethod {
    service: Arc<std::boxed::Box<dyn LifecycleServer + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for StopApplicationMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, StopApplicationRequest, stop_application);
    }
}

struct PrepareSystemFreezeMethod {
    service: Arc<std::boxed::Box<dyn LifecycleServer + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for PrepareSystemFreezeMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, PrepareFreezeRequest, prepare_system_freeze);
    }
}

struct PrepareSystemShutdownMethod {
    service: Arc<std::boxed::Box<dyn LifecycleServer + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for PrepareSystemShutdownMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, PrepareShutdownRequest, prepare_system_shutdown);
    }
}

struct ShutdownSystemMethod {
    service: Arc<std::boxed::Box<dyn LifecycleServer + Send + Sync>>,
}

#[async_trait]
impl ::ttrpc::r#async::MethodHandler for ShutdownSystemMethod {
    async fn handler(&self, ctx: ::ttrpc::r#async::TtrpcContext, req: ::ttrpc::Request) -> ::ttrpc::Result<(u32, Vec<u8>)> {
        ::ttrpc::async_request_handler!(self, ctx, req, application_interface, ShutdownRequest, shutdown_system);
    }
}

#[async_trait]
pub trait LifecycleServer: Sync {
    async fn get_applications(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::GetApplicationsRequest) -> ::ttrpc::Result<super::application_interface::GetApplicationsResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/get_applications is not supported".to_string())))
    }
    async fn get_application_status(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::GetApplicationStatusRequest) -> ::ttrpc::Result<super::application_interface::GetApplicationStatusResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/get_application_status is not supported".to_string())))
    }
    async fn start_application(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::StartApplicationRequest) -> ::ttrpc::Result<super::application_interface::StartApplicationResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/start_application is not supported".to_string())))
    }
    async fn restart_application(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::RestartApplicationRequest) -> ::ttrpc::Result<super::application_interface::RestartApplicationResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/restart_application is not supported".to_string())))
    }
    async fn pause_application(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::PauseApplicationRequest) -> ::ttrpc::Result<super::application_interface::PauseApplicationResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/pause_application is not supported".to_string())))
    }
    async fn resume_application(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::ResumeApplicationRequest) -> ::ttrpc::Result<super::application_interface::ResumeApplicationResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/resume_application is not supported".to_string())))
    }
    async fn stop_application(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::StopApplicationRequest) -> ::ttrpc::Result<super::application_interface::StopApplicationResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/stop_application is not supported".to_string())))
    }
    async fn prepare_system_freeze(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::PrepareFreezeRequest) -> ::ttrpc::Result<super::application_interface::PrepareFreezeResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/prepare_system_freeze is not supported".to_string())))
    }
    async fn prepare_system_shutdown(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::PrepareShutdownRequest) -> ::ttrpc::Result<super::application_interface::PrepareShutdownResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/prepare_system_shutdown is not supported".to_string())))
    }
    async fn shutdown_system(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: super::application_interface::ShutdownRequest) -> ::ttrpc::Result<super::application_interface::ShutdownResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.LifecycleServer/shutdown_system is not supported".to_string())))
    }
}

pub fn create_lifecycle_server(service: Arc<std::boxed::Box<dyn LifecycleServer + Send + Sync>>) -> HashMap <String, Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>> {
    let mut methods = HashMap::new();

    methods.insert("/grpc.LifecycleServer/get_applications".to_string(),
                    std::boxed::Box::new(GetApplicationsMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.LifecycleServer/get_application_status".to_string(),
                    std::boxed::Box::new(GetApplicationStatusMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.LifecycleServer/start_application".to_string(),
                    std::boxed::Box::new(StartApplicationMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.LifecycleServer/restart_application".to_string(),
                    std::boxed::Box::new(RestartApplicationMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.LifecycleServer/pause_application".to_string(),
                    std::boxed::Box::new(PauseApplicationMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.LifecycleServer/resume_application".to_string(),
                    std::boxed::Box::new(ResumeApplicationMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.LifecycleServer/stop_application".to_string(),
                    std::boxed::Box::new(StopApplicationMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.LifecycleServer/prepare_system_freeze".to_string(),
                    std::boxed::Box::new(PrepareSystemFreezeMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.LifecycleServer/prepare_system_shutdown".to_string(),
                    std::boxed::Box::new(PrepareSystemShutdownMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods.insert("/grpc.LifecycleServer/shutdown_system".to_string(),
                    std::boxed::Box::new(ShutdownSystemMethod{service: service.clone()}) as std::boxed::Box<dyn ::ttrpc::r#async::MethodHandler + Send + Sync>);

    methods
}
