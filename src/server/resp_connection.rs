use std::io;
use tokio::{
    io::{BufReader, BufWriter, AsyncWriteExt}, 
    net::TcpStream,
};

use kv_database::protocol::{
    resp::{
        decode::resp_decode, 
        encode::resp_encode,
    }, 
    reply::Reply,
};
use crate::{
    engine::SharedEngine, 
    server::connection::execute_tokens
};

pub(crate) async fn handle_resp_connection(stream: TcpStream, engine: SharedEngine) -> io::Result<()> {
    let (reader, writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut buf_writer = BufWriter::new(writer);

    loop {
        let tokens = match resp_decode(&mut buf_reader).await {
            Ok(Some(lines)) => lines, 
            Ok(None) => return Ok(()), 
            Err(e) => {
                // report error and close connection, since byte offset
                // is now unknown
                let reply = Reply::Error(format!("ERR protocol error: {e}"));
                let _ = buf_writer.write_all(&resp_encode(&reply)).await;
                return Ok(());
            }
        };
        let response = resp_encode(&execute_tokens(&engine, &tokens).await);
        buf_writer.write_all(&response).await?;

        // push replies only when read buffer is empty
        if buf_reader.buffer().is_empty() {
            buf_writer.flush().await?;
        }
    }
}
