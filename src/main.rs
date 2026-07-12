mod protocol;
mod server;
mod store;

fn main() {
    server::start_connection(100);
}
