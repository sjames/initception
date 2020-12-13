use tokio::io::{self, AsyncBufReadExt, ReadHalf};
use tokio::stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::{debug, error, info, Level};

use crate::common::TxHandle;
use crate::context::RuntimeEntityReference;
use crate::error::InitceptionServerError;



/// spawn a server to handle a service. The tx handle is used to send back messages to
/// the main task. The stream is used to communicate with the process.
/// the spanwnref gives you a shared reference to the launched service context
pub async fn manage_a_service(
    tx: TxHandle,
    stream: tokio::net::UnixStream,
    spawnref: RuntimeEntityReference,
) {
    info!("handle_receive!!!");

   // let (s1, s2) = std::os::unix::net::UnixStream::pair().unwrap();
    //let client = ApplicationInterfaceAsyncRPCClient::new(BincodeAsyncClientTransport::<_,_>>::new(stream));
    

    let (reader, writer) = tokio::io::split(stream);

    tokio::spawn(process_commands(spawnref, reader));

    //tokio::io::AsyncBufRead::lines(reader).await
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
