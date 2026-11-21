use std::io;
use tokio::{
    net::TcpStream,
    io::{BufReader, AsyncWriteExt},
};

use kv_database::protocol::{
    command::{parse_command, parse_tokens},
    reply::Reply, 
    ProtocolError,
    line::{encode::line_encode, decode::line_decode},
};
use crate::engine::SharedEngine;
use crate::dispatch::dispatch;

// returns response based on request
pub(crate) async fn handle_connection(stream: TcpStream, engine: SharedEngine) -> io::Result<()> {
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
        match line_decode(&mut buf_reader, &mut line_bytes).await {
            Ok(0) => break, // client closed connection without problems
            Ok(_) => {}
            Err(e) => {
                eprintln!("Read error: {e}");
                break;
            }
        }
        let response = execute_command(&engine, &line_bytes).await;
        if let Err(e) = writer.write_all(response.as_bytes()).await {
            eprintln!("Write error: {e}");
            break;
        }
    }
    Ok(())
}

pub(crate) async fn execute_command(engine: &SharedEngine, line: &[u8]) -> String {
    let trimmed = line.trim_ascii_end();
    let result = parse_command(trimmed);
    match result {
        Ok(cmd) => match line_encode(&dispatch(cmd, engine.clone()).await) {
            Ok(resp) => resp,
            Err(e) => format!("ERR internal encode failure: {e}\n"),
        },
        Err(ProtocolError::Empty) => "ERR empty command\n".to_string(),
        Err(ProtocolError::UnknownCommand(cmd)) => format!("ERR unknown command '{cmd}'\n"),
        Err(ProtocolError::UnknownKeyword(key)) => format!("ERR keyword '{key}' not recognized\n"),
        Err(ProtocolError::WrongArity) => "ERR wrong number of arguments\n".to_string(),
        Err(ProtocolError::WrongType(t)) => format!("ERR field '{t}' is invalid\n"),
        Err(ProtocolError::InvalidUtf8) => "ERR invalid UTF-8 in argument\n".to_string(),
    }
}

fn protocol_error_reply(e: ProtocolError) -> Reply {
    let msg = match e {
        ProtocolError::Empty => "ERR empty command".to_string(), 
        ProtocolError::UnknownCommand(cmd) => format!("ERR unknown command '{cmd}'"), 
        ProtocolError::UnknownKeyword(key) => format!("ERR keyword '{key}' not recognized"), 
        ProtocolError::WrongArity => "ERR wrong number of arguments".to_string(),
        ProtocolError::WrongType(t) => format!("ERR field '{t}' is invalid"),
        ProtocolError::InvalidUtf8 => "ERR invalid UTF-8 in argument".to_string(),
    };
    Reply::Error(msg)
}

pub(crate) async fn execute_tokens(engine: &SharedEngine, tokens: &[Vec<u8>]) -> Reply {
    match parse_tokens(tokens) {
        Ok(cmd) => dispatch(cmd, engine.clone()).await, 
        Err(e) => protocol_error_reply(e),
    }
}