use std::io;

mod protocol;
mod server;
mod engine;
mod store;
mod lru_cache;
mod sorted_set;
mod sorted_set_store;
mod wal;

#[tokio::main]
pub async fn main() -> io::Result<()> {
    server::start_connection().await
}
