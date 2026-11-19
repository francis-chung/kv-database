use std::{
    error::Error, io, sync::Arc,
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    io::{AsyncBufReadExt, BufReader, AsyncWriteExt},
    sync::Mutex,
};

use crate::{engine::Engine, snapshot::{SnapshotError, write_snapshot}};
use crate::store::Db;
use crate::protocol::{
    parse_command,
    Command,
    ProtocolError
};
use crate::wal::{
    encode_record,
    replay,
    truncate_wal,
    WriteAheadLog
};
use crate::snapshot::load_snapshot;
use crate::dispatch::{MutexEngine, dispatch};

// returns response based on request
pub(crate) async fn handle_connection(stream: TcpStream, engine: MutexEngine) -> Result<(), Box<dyn Error>> {
    // into_split used for looping while requesting and responding later
    // into_split consumes stream and uses Arc-like architecture
    // both halves can be used and mutated without Mutex
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
        let response = execute_command(&engine, std::str::from_utf8(&line_bytes).unwrap()).await;
        if let Err(e) = writer.write_all(response.as_bytes()).await {
            eprintln!("Write error: {e}");
            break;
        }
    }
    Ok(())
}

// execution for frontend 
// essentially handle_connection but simpler
pub(crate) async fn execute_command(engine: &MutexEngine, line: &str) -> String {
    let trimmed = line.trim_ascii_end();
    let result = parse_command(trimmed.as_bytes());
    match result {
        Ok(cmd) => match dispatch(cmd, engine.clone()).await {
            Ok(resp) => resp,
            Err(e) => format!("ERR {e}\n"),
        },
        Err(ProtocolError::Empty) => "ERR empty input\n".to_string(),
        Err(ProtocolError::UnknownCommand(cmd)) => format!("ERR command {cmd} not recognized\n"),
        Err(ProtocolError::UnknownKeyword(key)) => format!("ERR keyword {key} not recognized\n"),
        Err(ProtocolError::WrongArity) => "ERR wrong number of arguments\n".to_string(),
        Err(ProtocolError::WrongType(t)) => format!("ERR field '{t}' was wrong type\n"),
        Err(ProtocolError::InvalidUtf8) => "ERR non-UTF-8 character(s)\n".to_string(),
    }
}