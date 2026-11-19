use std::{
    io, 
    sync::Arc,
};
use tokio::{
    net::TcpListener,
    sync::Mutex,
};

use crate::engine::Engine;
use crate::store::Db;
use crate::wal::{
    replay,
    WriteAheadLog
};
use crate::snapshot::load_snapshot;
use crate::engine::MutexEngine;

pub mod connection;
mod maintenance;

use crate::server::connection::handle_connection;
use crate::server::maintenance::periodic_snapshot;

const ADDRESS: &str = "127.0.0.1:7878";
const LOG_PATH: &str = "src/files/log.txt";
const SNAPSHOT_PATH: &str = "src/files/snapshot.txt";

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