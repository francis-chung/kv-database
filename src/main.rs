mod protocol;
mod server;
mod engine;
mod store;
mod lru_cache;
mod sorted_set;
mod sorted_set_store;
mod wal;

fn main() {
    server::start_connection();
}
