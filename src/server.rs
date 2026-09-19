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

const ADDRESS: &str = "127.0.0.1:7878";
const LOG_PATH: &str = "src/files/log.txt";
const SNAPSHOT_PATH: &str = "src/files/snapshot.txt";

pub(crate) type MutexEngine = Arc<Mutex<Engine<tokio::fs::File>>>;

impl From<SnapshotError> for io::Error {
    fn from(error: SnapshotError) -> Self {
        match error {
            SnapshotError::Io(e) => e,
            SnapshotError::InvalidSnapshot => {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid snapshot format")
            }
            SnapshotError::UnexpectedEof => {
                io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected EOF in snapshot")
            }
        }
    }
}

// begins watching the address and delegating connection handling
pub async fn start_connection() -> io::Result<MutexEngine> {
    let listener = TcpListener::bind(ADDRESS).await?;

    let mut store = Db::new();

    let _snapshot_wal_pos = load_snapshot(SNAPSHOT_PATH, &mut store).await?;

    // replays all logs prior to starting
    // truncates log to longest well-formed prefix
    let good_len = replay(LOG_PATH, &mut store).await?;

    let file = std::fs::OpenOptions::new().write(true).open(LOG_PATH)?;
    file.set_len(good_len)?;
    // release std handle before reopening with async
    drop(file);
    let async_file = tokio::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(LOG_PATH)
        .await?;

    // initializes logger to continue appending to log
    let logger = WriteAheadLog::new(async_file).await?;
    let engine = Arc::new(Mutex::new(Engine {
        store,
        logger
    }));

    // begin snapshotting loop asynchronously
    let cloned_engine = Arc::clone(&engine);
    tokio::spawn(async move {
        if let Err(e) = periodic_snapshot(cloned_engine, SNAPSHOT_PATH, LOG_PATH).await {
            eprintln!("Periodic snapshot failed: {e:?}");
        }
    });

    // spawn the TCP server loop
    let tcp_engine = Arc::clone(&engine);
    tokio::spawn(async move {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error in stream: {e}");
                    continue;
                }
            };

            let cloned_engine = Arc::clone(&tcp_engine);
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, cloned_engine).await {
                    eprintln!("Failed to handle connection: {e:?}");
                }
            });
        }
    });

    Ok(engine)
}

// returns response based on request
async fn handle_connection(stream: TcpStream, engine: MutexEngine) -> Result<(), Box<dyn Error>> {
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
        let trimmed = line_bytes.trim_ascii_end();
        let result = parse_command(&trimmed);
        let response = match result {
            Ok(cmd) => {
                match dispatch(cmd, engine.clone()).await {
                    Ok(resp) => resp,
                    Err(e) => format!("ERR {e}\n")
                }
            }
            Err(ProtocolError::Empty) => {
                "ERR empty input\n".to_string()
            }
            Err(ProtocolError::UnknownCommand(cmd)) => {
                format!("ERR command {cmd} not recognized\n")
            }
            Err(ProtocolError::UnknownKeyword(key)) => {
                format!("ERR keyword {key} not recognized\n")
            }
            Err(ProtocolError::WrongArity) => {
                "ERR wrong number of arguments\n".to_string()
            }
            Err(ProtocolError::WrongType(t)) => {
                format!("ERR field '{t}' was wrong type\n")
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

async fn dispatch(cmd: Command, engine: MutexEngine) -> io::Result<String> {
    match cmd {
        Command::Get { key } => {
            let mut eng = engine.lock().await;
            match eng.store.kv_store.get(&key) {
                Some(value) => Ok(format!("VALUE {value}\n")),
                None => Ok("NIL\n".to_string())
            }
        }
        Command::Set { key, value } => {
            let mut eng = engine.lock().await;
            let bytes = encode_record(&Command::Set { key: key.clone(), value: value.clone() });
            eng.logger.buffered_log(&bytes).await?;
            eng.store.kv_store.insert(key, value);
            Ok("OK\n".to_string())
        }
        Command::Del { key } => {
            let mut eng = engine.lock().await;
            let bytes = encode_record(&Command::Del { key: key.clone() });
            eng.logger.buffered_log(&bytes).await?;
            eng.store.kv_store.remove(&key);
            Ok("OK\n".to_string())
        }
        Command::Exists { key } => {
            let mut eng = engine.lock().await;
            match eng.store.kv_store.contains_key(&key) {
                true => Ok("1\n".to_string()),
                false => Ok("0\n".to_string())
            }
        }
        Command::DbSize => {
            let result = engine.lock().await.store.kv_store.len();
            Ok(format!("{result}\n"))
        }
        Command::Clear => {
            engine.lock().await.store.kv_store.clear();
            Ok("OK\n".to_string())
        }
        Command::Zadd { key, member, score } => {
            let mut eng = engine.lock().await;
            let bytes = encode_record(&Command::Zadd { key: key.clone(), member: member.clone(), score });
            eng.logger.buffered_log(&bytes).await?;
            eng.store.sorted_sets.zadd(&key, member, score);
            Ok("OK\n".to_string())
        }
        Command::Zscore { key, member } => {
            let eng = engine.lock().await;
            match eng.store.sorted_sets.zscore(&key, &member) {
                Some(value) => Ok(format!("VALUE {value}\n")),
                None => Ok("NIL\n".to_string())
            }
        }
        Command::Zrem { key, member } => {
            let mut eng = engine.lock().await;
            let bytes = encode_record(&Command::Zrem { key: key.clone(), member: member.clone() });
            eng.logger.buffered_log(&bytes).await?;
            eng.store.sorted_sets.zrem(&key, &member);
            Ok("OK\n".to_string())
        }
        Command::Zrange { key, from, to, with_scores } => {
            let mut response = String::new();
            let eng = engine.lock().await;
            if let Some(rows) = eng.store.sorted_sets.zrange(&key, from, to, with_scores) {
                for (index, (key, poss_value)) in rows.iter().enumerate() {
                    if let Some(val) = poss_value {
                        response += &format!("{}) \"{key}\"\n", 2 * index + 1);
                        response += &format!("{}) \"{val}\"\n", 2 * index + 2);
                    } else {
                        response += &format!("{}) \"{key}\"\n", index + 1);
                    }
                }
            } else {
                response = "NIL\n".to_string();
            }
            Ok(response)
        }
    }
}

async fn periodic_snapshot(engine: MutexEngine, snapshot_path: &str, wal_path: &str) -> io::Result<()> {
    let temp_ss_path = snapshot_path.replace(".txt", ".tmp");
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        let cloned_engine = Arc::clone(&engine);
        write_snapshot(&cloned_engine.lock().await.store, &temp_ss_path).await?;
        std::fs::rename(&temp_ss_path, &snapshot_path)?;
        truncate_wal(wal_path).await?;
    }
}