use std::{
    io, 
    sync::Arc,
};
use tokio::{
    net::{TcpListener, TcpStream},
};

use crate::engine::Engine;
use crate::store::Db;
use crate::wal::{
    replay,
    WriteAheadLog,
};
use crate::snapshot::load_snapshot;
use crate::engine::SharedEngine;

pub mod connection;
mod resp_connection;
mod maintenance;

use crate::server::{
    connection::handle_connection, 
    resp_connection::handle_resp_connection, 
    maintenance::{periodic_flush, periodic_snapshot},
};

const RESP_ADDRESS: &str = "127.0.0.1:7878";
const LINE_ADDRESS: &str = "127.0.0.1:7879";
const LOG_PATH: &str = "src/files/log.txt";
const SNAPSHOT_PATH: &str = "src/files/snapshot.txt";

async fn accept_loop<F, Fut>(listener: TcpListener, engine: SharedEngine, handler: F)
where 
    F: Fn(TcpStream, SharedEngine) -> Fut + Send + 'static, 
    Fut: Future<Output = io::Result<()>> + Send + 'static,
{
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let fut = handler(stream, Arc::clone(&engine));
                tokio::spawn(async move {
                    if let Err(e) = fut.await {
                        eprintln!("Connection error: {e}");
                    }
                });
            }, 
            Err(e) => eprintln!("Error accepting connection: {e}"),
        }
    }
}

// begins watching the address and delegating connection handling
pub async fn start_connection() -> io::Result<SharedEngine> {
    let resp_listener = TcpListener::bind(RESP_ADDRESS).await?;
    let line_listener = TcpListener::bind(LINE_ADDRESS).await?;

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
    let engine = Arc::new(Engine::new(store, logger));

    // begin flushing WALs and snapshotting loop asynchronously
    let cloned_engine = Arc::clone(&engine);
    tokio::spawn(async move {
        periodic_flush(cloned_engine).await;
    });
    let cloned_engine = Arc::clone(&engine);
    tokio::spawn(async move {
        if let Err(e) = periodic_snapshot(cloned_engine, SNAPSHOT_PATH, LOG_PATH).await {
            eprintln!("Periodic snapshot failed: {e:?}");
        }
    });

    // spawn the TCP server loop
    tokio::spawn(accept_loop(resp_listener, Arc::clone(&engine), handle_resp_connection));
    tokio::spawn(accept_loop(line_listener, Arc::clone(&engine), handle_connection));

    Ok(engine)
}