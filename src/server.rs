use std::{
    sync::{Arc, Mutex},
    error::Error
};
use tokio::{
    net::{TcpListener, TcpStream}, 
    io::{AsyncBufReadExt, BufReader, AsyncWriteExt}
};

use crate::store::HashMapWrapper;
use crate::protocol::{
    parse_command, 
    Command, 
    ProtocolError
};

const ADDRESS: &str = "127.0.0.1:7878";

type Store = Arc<Mutex<HashMapWrapper<String, String>>>;

// begins watching the address and delegating connection handling
#[tokio::main]
pub async fn start_connection() {
    let listener = match TcpListener::bind(ADDRESS).await {
        Ok(sock) => sock, 
        Err(e) => {
            eprintln!("Could not bind to {ADDRESS}: {e}");
            return;
        }
    };
    
    let store = Arc::new(Mutex::new(HashMapWrapper::<String, String>::new()));
    
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error in stream: {e}");
                continue;
            }
        };

        let store = Arc::clone(&store);
        let _ = tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, store).await {
                eprintln!("Failed to handle connection: {e}");
            }
        });
    }
}

// returns response based on request
async fn handle_connection(mut stream: TcpStream, store: Store) -> Result<(), Box<dyn Error>> {
    // into_split used for looping while requesting and responding later
    // into_split consumes stream and uses Arc-like architecture
    let (reader, mut writer) = stream.into_split();
    // enables async buffering 
    let mut buf_reader = BufReader::new(reader);
    // byte vector allows non-UTF-8 characters, handled later
    let mut line_bytes = Vec::new();

    loop {
        line_bytes.clear();
        match buf_reader.read_until(b'\n', &mut line_bytes).await {
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
        if let Err(e) = writer.write_all(response.as_bytes()).await {
            eprintln!("Write error: {e}");
            break;
        }
    }
    Ok(())
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
