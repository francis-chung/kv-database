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

    let store = Arc::new(Mutex::new(HashMapWrapper::<String, String>::new()));
    
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s, 
            Err(e) => {
                eprintln!("Error in stream (accepted): {e}");
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
fn handle_connection<K, V>(mut stream: TcpStream, store: Arc<Mutex<HashMapWrapper<K, V>>>) 
where 
    K: Eq + std::hash::Hash, 
    V: Clone
{
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

        let result = parse_command(line.trim_end());
        let response = match result {
            Ok(cmd) => {
                dispatch(cmd, &store)
            }
            Err(ProtocolError::Empty) => {
                "ERR empty input\n".to_string()
            }
            Err(ProtocolError::UnknownCommand(cmd)) => {
                "ERR command {cmd} not recognized\n".to_string()
            }
            Err(ProtocolError::WrongArity) => {
                "ERR too many arguments\n".to_string()
            }
        };
        if let Err(e) = writer.write_all(response.as_bytes()) {
            eprintln!("Write error: {e}");
            break;
        }
    }
}

fn dispatch<K, V>(cmd: Command, store: &Arc<Mutex<HashMapWrapper<K, V>>>) -> String 
where 
    K: Eq + std::hash::Hash, 
    V: Clone
{
    match cmd {
        Command::Get { key } => {
            let map = store.lock().unwrap();
            match map.get(&key) {
                Some(value) => format!("VALUE {}\n", value.to_string()), 
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
    }
}
