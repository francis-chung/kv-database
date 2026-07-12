use std::{
    sync::{Arc, Mutex},
    io::{BufReader, prelude::*}, 
    net::{TcpListener, TcpStream}
};

use kv_database::ThreadPool;
use crate::store::HashMapWrapper;
use crate::protocol::{
    parse_command, 
    Command, 
    ProtocolError
};

const ADDRESS: &str = "127.0.0.1:7878";

type Store = Arc<Mutex<HashMapWrapper<String, String>>>;

// begins watching the address and delegating connection handling
pub fn start_connection(threads: usize) {
    let listener = match TcpListener::bind(ADDRESS) {
        Ok(sock) => sock, 
        Err(e) => {
            eprintln!("Could not bind to {ADDRESS}: {e}");
            return;
        }
    };
    let pool = ThreadPool::new(threads);

    let store = Arc::new(Mutex::new(HashMapWrapper::<String, String>::new()));
    
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s, 
            Err(e) => {
                eprintln!("Error in stream: {e}");
                continue;
            }
        };
        
        let store = Arc::clone(&store);
        pool.execute(|| {
            handle_connection(stream, store);
        });
    }

    println!("Shutting down.");
}

// returns response based on request
fn handle_connection(mut stream: TcpStream, store: Store) {
    // try_clone used for looping while requesting and responding later
    let mut reader = BufReader::new(
        stream
            .try_clone()
            .unwrap_or_else(|e| panic!("Failed to clone TCP stream for connection handling: {e}")),
    );
    let mut writer = stream;
    // byte vector allows non-UTF-8 characters, handled later
    let mut line_bytes = Vec::new();

    loop {
        line_bytes.clear();
        match reader.read_until(b'\n', &mut line_bytes) {
            Ok(0) => break, // client closed connection without problems
            Ok(_) => {}
            Err(e) => {
                eprintln!("Read error: {e}");
                break;
            }
        }

        let trimmed = line_bytes.trim_ascii_end();
        let result = parse_command(&trimmed);
        let response = match result {
            Ok(cmd) => {
                dispatch(cmd, &store)
            }
            Err(ProtocolError::Empty) => {
                "ERR empty input\n".to_string()
            }
            Err(ProtocolError::UnknownCommand(cmd)) => {
                format!("ERR command {cmd} not recognized\n")
            }
            Err(ProtocolError::WrongArity) => {
                "ERR wrong number of arguments\n".to_string()
            }
            Err(ProtocolError::InvalidUtf8) => {
                "ERR non-UTF-8 character(s)\n".to_string()
            }
        };
        if let Err(e) = writer.write_all(response.as_bytes()) {
            eprintln!("Write error: {e}");
            break;
        }
    }
}

fn dispatch(cmd: Command, store: &Store) -> String {
    match cmd {
        Command::Get { key } => {
            let map = store.lock().unwrap();
            match map.get(&key) {
                Some(value) => format!("VALUE {value}\n"), 
                None => "NIL\n".to_string()
            }
        }
        Command::Set { key, value } => {
            store.lock().unwrap().insert(key, value);
            "OK\n".to_string()
        }
        Command::Del { key } => {
            store.lock().unwrap().remove(&key);
            "OK\n".to_string()
        }
        Command::Exists { key } => {
            let map = store.lock().unwrap();
            match map.contains_key(&key) {
                true => "1\n".to_string(), 
                false => "0\n".to_string()
            }
        }
        Command::DbSize => {
            let result = store.lock().unwrap().len();
            format!("{result}\n")
        }
        Command::Clear => {
            store.lock().unwrap().clear();
            "OK\n".to_string()
        }
    }
}
