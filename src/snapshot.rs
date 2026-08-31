use crc32fast::Hasher;
use ordered_float::OrderedFloat;
use kv_database::protocol::ProtocolError::InvalidUtf8;
use kv_database::wal::WalError::{self, ChecksumMismatch, UnexpectedEof, UnknownCommandByte};
use std::io::{self, Write, Read, Cursor};
use std::fs;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

use crate::store::Db;
use crate::wal::{write_bytes_with_len, read_string, read_float};

// prefix to verify this is supposed to be a snapshot
const SNAPSHOT_MAGIC: u32 = 0x534E4150;
// version for potential future modifications to format
const SNAPSHOT_VERSION: u32 = 1;

#[derive(Debug)]
pub enum SnapshotError {
    Io(io::Error), 
    InvalidSnapshot, 
    UnexpectedEof,
}

impl From<WalError> for SnapshotError {
    fn from(error: WalError) -> Self {
        match error {
            WalError::Io(e) => SnapshotError::Io(e), 
            WalError::UnexpectedEof => SnapshotError::UnexpectedEof, 
            _ => SnapshotError::InvalidSnapshot, // the other cases won't happen so i'm putting them here now
        }
    }
}

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

pub fn load_snapshot(path: &str, db: &mut Db) -> Result<u64, SnapshotError> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b, 
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0), 
        Err(e) => return Err(SnapshotError::Io(e)) // FIX
    };
    load_snapshot_from_bytes(&bytes, db)
}

pub fn load_snapshot_from_bytes(bytes: &[u8], db: &mut Db) -> Result<u64, SnapshotError> {
    let total_len = bytes.len();
    let cursor = &mut &bytes[..];

    // allows for returning errors earlier without losing snapshot length
    let result: Result<(), SnapshotError> = (|| {
        let magic = read_u32(cursor)?;
        if magic != SNAPSHOT_MAGIC {
            return Err(SnapshotError::InvalidSnapshot);
        }
        let version = read_u32(cursor)?;
        if version != SNAPSHOT_VERSION {
            return Err(SnapshotError::InvalidSnapshot);
        }
        let _timestamp = read_u32(cursor)?;
        
        let kv_cnt = read_u32(cursor)?;
        for _ in 0..kv_cnt {
            let key = read_string(cursor)?;
            let val = read_string(cursor)?;
            db.kv_store.insert(key, val);
        }
        let ss_cnt = read_u32(cursor)?;
        for _ in 0..ss_cnt {
            let key = read_string(cursor)?;
            let member_count = read_u32(cursor)?;
            for _ in 0..member_count {
                let member = read_string(cursor)?;
                let score = OrderedFloat(read_float(cursor)?);
                db.sorted_sets.zadd(&key, member, score);
            }
        }
        Ok(())
    })();
    
    let bytes_processed = (total_len - cursor.len()) as u64;
    match result {
        Ok(()) => Ok(bytes_processed), 
        Err(e) => {
            eprintln!("Snapshot load failed after {bytes_processed} bytes: {e:?}");
            Err(e)
        }
    }
}

fn read_u32(cursor: &mut &[u8]) -> Result<u32, SnapshotError> {
    if cursor.len() < 4 {
        return Err(SnapshotError::UnexpectedEof);
    }
    let res = u32::from_le_bytes(cursor[..4].try_into().unwrap());
    *cursor = &cursor[4..];
    Ok(res)
}