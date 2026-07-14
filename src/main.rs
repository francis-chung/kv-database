mod protocol;
mod server;
mod store;
mod lru_cache;
mod skip_list;

fn main() {
    server::start_connection();
}
