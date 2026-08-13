use crate::protocol::Command;
use crate::store::Db;
use crc32fast::Hasher;
use std::{
    io::{self, Read, Cursor},
    pin::Pin, 
    task::{Context, Poll},
};
use tokio::{
    fs::{File, OpenOptions}, 
    io::{BufWriter, AsyncWrite, AsyncWriteExt},
};

const CMD_SET: u8 = 1;
const CMD_DEL: u8 = 2;

pub struct WriteAheadLog<W: AsyncWrite + Unpin> {
    writer: BufWriter<W>
}

impl<W: AsyncWrite + Unpin> WriteAheadLog<W> {
    pub async fn new(writer: W) -> io::Result<Self> {
        Ok(WriteAheadLog { writer: BufWriter::new(writer) })
    }
    
    // writes record to buffer, then writes to disk
    pub async fn buffered_log(&mut self, record: &[u8]) -> io::Result<()> {
        self.writer.write_all(record).await?;
        self.writer.flush().await?;
        // forces OS to write buffered content to disk before returning
        // self.writer.get_ref().sync_all().await?;
        Ok(())
    }
}

pub fn replay(path: &str, store: &mut Db) -> io::Result<u64> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b, 
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0), 
        Err(e) => return Err(e),
    };
    // Cursor facilitates passing byte array into func requiring Read type
    let mut cursor = Cursor::new(bytes);
    // determines until which position the byte array is still valid / not malformed
    let mut good_len: u64 = 0;

    loop {
        let pos_before = cursor.position();
        match decode_record(&mut cursor) {
            Ok(Some(cmd)) => {
                apply_to_store(store, cmd);
                good_len = cursor.position();
            } 
            Ok(None) => break, 
            Err(e) => {
                good_len = pos_before;
                match e {
                    WalError::ChecksumMismatch | WalError::UnexpectedEof => {
                        eprintln!("Malformed command in log, stopping replay at position {good_len}");
                        break;
                    }
                    WalError::UnknownCommandByte(b) => {
                        eprintln!("Unknown command {b} in log, stopping replay at position {good_len}");
                        break;
                    }
                    WalError::Io(e) => return Err(e), 
                    WalError::InvalidUtf8 => {
                        eprintln!("Invalid UTF-8 in log, stopping replay at position {good_len}");
                        break;
                    }
                }
            }
        }
    }
    Ok(good_len)
}

fn apply_to_store(store: &mut Db, cmd: Command) {
    match cmd {
        Command::Set { key, value } => {
            store.kv_store.insert(key, value);
        }
        Command::Del { key } => {
            store.kv_store.remove(&key);
        }
        _ => eprintln!("Improper command present in log")
    }
}

#[derive(Debug)]
pub enum WalError {
    Io(io::Error), 
    ChecksumMismatch, 
    UnknownCommandByte(u8), 
    InvalidUtf8, 
    UnexpectedEof,
}

impl From<io::Error> for WalError {
    fn from(e: io::Error) -> Self {
        WalError::Io(e)
    }
}

// WAL record is checksummed and self-delimiting
// format: [checksum: u32][record_len: u32][cmd_type: u8]...
// [key_len: u32][key_bytes][val_len: u32][val_bytes]
// uses little-endianness
pub fn encode_record(cmd: &Command) -> Vec<u8> {
    let mut body = Vec::new();

    match cmd {
        Command::Set { key, value } => {
            body.push(CMD_SET);
            write_bytes_with_len(&mut body, key.as_bytes());
            write_bytes_with_len(&mut body, value.as_bytes());
        }
        Command::Del { key } => {
            body.push(CMD_DEL);
            write_bytes_with_len(&mut body, key.as_bytes());
        }
        _ => panic!("Command not loggable in WAL") // TODO: handle error execution properly
    }

    let mut hasher = Hasher::new();
    hasher.update(&body);
    let checksum = hasher.finalize();

    let mut record = Vec::with_capacity(4 + 4 + body.len());
    record.extend_from_slice(&checksum.to_le_bytes());
    record.extend_from_slice(&(body.len() as u32).to_le_bytes());
    record.extend_from_slice(&body);
    record
}

// adds char slice into vector buffer in WAL record format
fn write_bytes_with_len(buf: &mut Vec<u8>, data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
    buf.extend_from_slice(data);
}

pub fn decode_record<R: Read>(reader: &mut R) -> Result<Option<Command>, WalError> {
    let mut checksum_buf = [0u8; 4];
    if let Err(e) = reader.read_exact(&mut checksum_buf) {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            return Ok(None); // clean end of file
        }
        return Err(e.into());
    }
    let expected_checksum = u32::from_le_bytes(checksum_buf);

    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).map_err(|_| WalError::UnexpectedEof)?;
    let body_len = u32::from_le_bytes(len_buf) as usize;

    let mut body = vec![0u8; body_len];
    reader.read_exact(&mut body).map_err(|_| WalError::UnexpectedEof)?;

    // recompute checksum over body to detect hidden corruption
    let mut hasher = Hasher::new();
    hasher.update(&body);
    if hasher.finalize() != expected_checksum {
        return Err(WalError::ChecksumMismatch);
    }

    let mut cursor = &body[..];
    let cmd_type = read_u8(&mut cursor)?;
    let cmd = match cmd_type {
        CMD_SET => {
            let key = read_string(&mut cursor)?;
            let value = read_string(&mut cursor)?;
            Command::Set { key, value }
        }
        CMD_DEL => {
            let key = read_string(&mut cursor)?;
            Command::Del { key }
        }
        other => return Err(WalError::UnknownCommandByte(other))
    };
    Ok(Some(cmd))
}

// reads one byte, advances slice reference and returns byte
fn read_u8(cursor: &mut &[u8]) -> Result<u8, WalError> {
    let b = *cursor.first().ok_or(WalError::UnexpectedEof)?;
    *cursor = &cursor[1..];
    Ok(b)
}

// same as read_u8 but reads 4 bytes
fn read_string(cursor: &mut &[u8]) -> Result<String, WalError> {
    if cursor.len() < 4 {
        return Err(WalError::UnexpectedEof);
    }
    let len = u32::from_le_bytes(cursor[..4].try_into().unwrap()) as usize;
    *cursor = &cursor[4..];
    if cursor.len() < len {
        return Err(WalError::UnexpectedEof);
    }
    let s = String::from_utf8(cursor[..len].to_vec()).map_err(|_| WalError::InvalidUtf8)?;
    *cursor = &cursor[len..];
    Ok(s)
}