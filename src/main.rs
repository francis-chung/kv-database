mod store;
mod server;

use store::HashMapWrapper;

fn main() {
    server::start_connection();
    let map = HashMapWrapper::<String, String>::new();
}
