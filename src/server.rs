use tokio::io::{self, AsyncBufReadExt, ReadHalf};
use tokio::stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::{debug, error, info, Level};

use crate::common::TxHandle;
use crate::context::RuntimeEntityReference;
use crate::error::InitceptionServerError;

use crate::application::src_gen::application_interface_ttrpc;
use crate::application::src_gen::application_interface;

use ttrpc::r#async::Server;
use ttrpc;
use async_trait::async_trait;

use std::sync::Arc;

struct ServiceManager 
{
    spawnref: RuntimeEntityReference, // reference to the runtime entity for this server
}

impl ServiceManager {
    fn new(spawnref: RuntimeEntityReference) -> Self {
        ServiceManager {
            spawnref,
        }
    }
}

#[async_trait]
impl application_interface_ttrpc::ApplicationManager for ServiceManager {

    async fn heartbeat(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: application_interface::HeartbeatRequest) -> ::ttrpc::Result<application_interface::HeartbeatReply> {
        
        Err(ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationManager/heartbeat is not supported".to_string())))
    }
    async fn statechanged(&self, _ctx: &::ttrpc::r#async::TtrpcContext, _req: application_interface::StateChangedRequest) -> ::ttrpc::Result<application_interface::StateChangedReply> {

        Err(ttrpc::Error::RpcStatus(::ttrpc::get_status(::ttrpc::Code::NOT_FOUND, "/grpc.ApplicationManager/statechanged is not supported".to_string())))
    }
}

/// spawn a server to handle a service. The tx handle is used to send back messages to
/// the main task. The stream is used to communicate with the process.
/// the spanwnref gives you a shared reference to the launched service context
pub async fn manage_a_service(
    tx: TxHandle,
    socket_name : String,
    spawnref: RuntimeEntityReference,
) {
    info!("handle_receive!!!");

    let service = Box::new(ServiceManager::new(spawnref)) as Box<dyn application_interface_ttrpc::ApplicationManager + Send + Sync>;
    let service = Arc::new(service);
    let service = application_interface_ttrpc::create_application_manager(service);

    let socket_name = String::from("unix://") + &socket_name;
    
    let mut server = Server::new().bind(&socket_name).unwrap().register_service(service);
    server.start().await.unwrap();
}

async fn process_commands(
    spawnref: RuntimeEntityReference,
    reader: ReadHalf<tokio::net::UnixStream>,
) -> std::result::Result<(), InitceptionServerError> {
    println!("reader ready - waiting for commands!");

    let mut lines = FramedRead::new(reader, LinesCodec::new());

    while let Some(line) = lines.next().await {
        println!("received line: {}", line.unwrap());
    }

    Ok(())
}
