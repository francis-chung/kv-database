use std::{
    io::{BufReader, prelude::*}, 
    net::{TcpListener, TcpStream}
};

use kv_database::ThreadPool;

const ADDRESS: &str = "127.0.0.1:7878";

// begins watching the address and delegating connection handling
pub fn start_connection() {
    let listener = match TcpListener::bind(ADDRESS) {
        Ok(sock) => sock, 
        Err(e) => {
            eprintln!("Could not bind to {ADDRESS}: {e}");
            return;
        }
    };
    let pool = ThreadPool::new(4);
    
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s, 
            Err(e) => {
                eprintln!("Error in stream (accepted): {e}");
                continue;
            }
        };
        
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

// returns response based on request
fn handle_connection(mut stream: TcpStream) {
    // try_clone used for looping while requesting and responding later
    let mut reader = BufReader::new(stream.try_clone().expect("Clone failed"));
    let mut writer = stream;
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // client closed connection without problems
            Ok(_) => {}
            Err(e) => {
                eprintln!("Read error: {e}");
                break;
            }
        }

        // IMPLEMENT: HANDLE_COMMAND
        let response = handle_command(line.trim_end());
        if let Err(e) = writer.write_all(response.as_bytes()) {
            eprintln!("Write error: {e}");
            break;
        }
    }
}

fn handle_command(query: &str) -> String {
    let response = format!("Placeholder. Query: {query}.");
    response
}