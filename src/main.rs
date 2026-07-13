mod protocol;
mod server;
mod store;
mod lru_cache;

fn main() {
    server::start_connection();
}
