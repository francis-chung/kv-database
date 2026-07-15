mod protocol;
mod server;
mod store;
mod lru_cache;
mod sorted_set;

fn main() {
    server::start_connection();
}
