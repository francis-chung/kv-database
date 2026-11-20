use std::{
    io, 
    sync::Arc, 
    time::Duration,
};

use crate::snapshot::write_snapshot;
use crate::wal::truncate_wal;
use crate::engine::MutexEngine;

pub(crate) async fn periodic_snapshot(engine: MutexEngine, snapshot_path: &str, wal_path: &str) -> io::Result<()> {
    let temp_ss_path = snapshot_path.replace(".txt", ".tmp");
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        let cloned_engine = Arc::clone(&engine);
        write_snapshot(&cloned_engine.lock().await.store, &temp_ss_path).await?;
        std::fs::rename(&temp_ss_path, snapshot_path)?;
        truncate_wal(wal_path).await?;
    }
}