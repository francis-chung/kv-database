use std::io;
use std::sync::Arc;

// public API from our library
use kv_database::protocol;
use kv_database::store;
use kv_database::wal;

// private modules only used by this binary
mod server;
mod engine;
mod snapshot;
mod http;

#[tokio::main]
pub async fn main() -> io::Result<()> {
    let engine = server::start_connection().await?;
    let shared_engine = Arc::clone(&engine);

    // start HTTP server on a different port
    let http_addr = "127.0.0.1:8080";
    let http_task = tokio::spawn(async move {
        if let Err(e) = http::start_http_server(shared_engine, http_addr).await {
            eprintln!("HTTP server error: {e}");
        }
    });

    // TCP server runs in a background task from start_connection()
    http_task.await?;
    Ok(())
}