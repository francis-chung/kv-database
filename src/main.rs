use std::io;

// public API from our library
use kv_database::protocol;
use kv_database::store;
use kv_database::wal;

// private modules only used by this binary
mod server;
mod engine;
mod snapshot;

#[tokio::main]
pub async fn main() -> io::Result<()> {
    server::start_connection().await
}
