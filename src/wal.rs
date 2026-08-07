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

