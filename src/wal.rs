use crate::protocol::Command;
use crc32fast::Hasher;
use std::io::{self, Read, Write};

const CMD_SET: u8 = 1;
const CMD_DEL: u8 = 2;

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
            Command::Set{ key, value }
        }
        CMD_DEL => {
            let key = read_string(&mut cursor)?;
            Command::Del{ key }
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
    OK(s)
}