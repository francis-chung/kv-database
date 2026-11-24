use std::{
    io, 
    time::Duration,
};
use tokio::io::AsyncWriteExt;

use crate::snapshot::write_snapshot;
use crate::engine::SharedEngine;

pub(crate) async fn periodic_flush(engine: SharedEngine) {
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    loop {
        ticker.tick().await;
        // a single failed flush should not stop future flushes
        let mut eng = engine.logger.lock().await;
        if let Err(e) = eng.writer.flush().await {
            eprintln!("WAL flush failed: {e}");
        }
        if let Err(e) = eng.writer.get_ref().sync_data().await {
            eprintln!("Write to disk failed: {e}");
        }
    }
}

pub(crate) async fn periodic_snapshot(engine: SharedEngine, snapshot_path: &str) -> io::Result<()> {
    let temp_ss_path = snapshot_path.replace(".txt", ".tmp");
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        // lock WAL first so SETs cannot append until truncate is done
        let mut logger = engine.logger.lock().await;
        logger.writer.flush().await?;
        let snapshot_data = {
            let store = engine.store.lock()
                              .map_err(|_| io::Error::new(io::ErrorKind::Other, "Store lock poisoned during snapshot"))?;
            store.clone()
        };
        write_snapshot(&snapshot_data, &temp_ss_path).await?;
        tokio::fs::rename(&temp_ss_path, snapshot_path).await?;
        // truncate logger file in place while logger still has a handler
        logger.truncate().await?;
    }
}
