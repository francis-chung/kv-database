use crc32fast::Hasher;
use std::io::{self, Write, Read, Cursor};
use std::fs;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

use crate::store::Db;
use crate::wal::write_bytes_with_len;

// prefix to verify this is supposed to be a snapshot
const SNAPSHOT_MAGIC: u32 = 0x534E4150;
// version for potential future modifications to format
const SNAPSHOT_VERSION: u32 = 1;

pub async fn write_snapshot(db: &Db, wal_position: u64, path: &str) -> io::Result<()> {
    let mut file = File::create(path).await?;
    let mut buf = Vec::new();

    // header with some metadata
    buf.extend_from_slice(&SNAPSHOT_MAGIC.to_le_bytes());
    buf.extend_from_slice(&SNAPSHOT_VERSION.to_le_bytes());
    buf.extend_from_slice(&chrono::Utc::now().timestamp_millis().to_le_bytes());
    buf.extend_from_slice(&wal_position.to_le_bytes());

    // key-value database and sorted sets adhere to files/snapshot.bin format
    let kv_count = db.kv_store.map.len() as u32;
    buf.extend_from_slice(&kv_count.to_le_bytes());
    for (key, value) in &db.kv_store.map {
        write_bytes_with_len(&mut buf, key.as_bytes());
        write_bytes_with_len(&mut buf, value.as_bytes());
    }

    let mut ss_count: u32 = 0;
    for (_, list) in &db.sorted_sets.sets {
        ss_count += list.len() as u32;
    }
    buf.extend_from_slice(&ss_count.to_le_bytes());
    for (key, list) in &db.sorted_sets.sets {
        write_bytes_with_len(&mut buf, key.as_bytes());
        let members = list.iter_all();
        buf.extend_from_slice(&(members.len() as u32).to_le_bytes());
        for (member, score) in members {
            write_bytes_with_len(&mut buf, member.as_bytes());
            buf.extend_from_slice(&score.0.to_le_bytes());
        }
    }

    let mut hasher = Hasher::new();
    hasher.update(&buf);
    let checksum = hasher.finalize();
    buf.extend_from_slice(&checksum.to_le_bytes());

    file.write_all(&buf).await?;
    file.sync_all().await?;
    Ok(())
}

pub fn load_snapshot_from_bytes(bytes: &[u8], db: &mut Db) -> io::Result<u64> {
    // TODO
    Ok(0)
}

pub fn load_snapshot(path: &str, db: &mut Db) -> io::Result<u64> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b, 
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0), 
        Err(e) => return Err(e)
    };
    load_snapshot_from_bytes(&bytes, db)
}