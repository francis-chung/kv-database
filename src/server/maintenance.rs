use std::{
    io, 
    time::Duration,
};

use crate::snapshot::write_snapshot;
use crate::wal::truncate_wal;
use crate::engine::SharedEngine;

pub(crate) async fn periodic_snapshot(engine: SharedEngine, snapshot_path: &str, wal_path: &str) -> io::Result<()> {
    let temp_ss_path = snapshot_path.replace(".txt", ".tmp");
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        // lock WAL first so SETs cannot append until truncate is done
        let _logger = engine.logger.lock().await;
        let store = engine.store.lock().await;
        write_snapshot(&store, &temp_ss_path).await?;
        drop(store);
        std::fs::rename(&temp_ss_path, snapshot_path)?;
        truncate_wal(wal_path).await?;
    }
}