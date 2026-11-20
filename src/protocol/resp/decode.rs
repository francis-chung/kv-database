use std::io;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

pub async fn resp_decode<R>(reader: &mut R) -> io::Result<Option<Vec<Vec<u8>>>>
where 
    R: AsyncBufRead + Unpin,
{
    let mut line_bytes = Vec::new();
    reader.read_until(b'\n', &mut line_bytes).await?;
    if line_bytes.len() == 0 {
        return Ok(None)
    }
    let stripped = line_bytes.strip_suffix(b"\r\n").unwrap();
    if stripped[0] != b'*' {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "first character must be '*'"));
    }
    let Ok(stripped_utf8) = str::from_utf8(stripped) else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "input must be UTF-8"));
    };
    let Ok(length) = stripped_utf8.parse::<usize>() else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "length must be an integer"));
    };
    let mut result = Vec::new();
    for _ in 0..length {
        reader.read_until(b'\n', &mut line_bytes).await?;
        if line_bytes.len() == 0 {
            break;
        }
        let stripped = line_bytes.strip_suffix(b"\r\n").unwrap();
        if stripped[0] != b'$' {
            return Err(io::Error::new(io::ErrorKind::InvalidData, 
                                      "first character of array element must be '$'"));
        }
        let Ok(stripped_utf8) = str::from_utf8(stripped) else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "input must be UTF-8"));
        };
        let Ok(inner_length) = stripped_utf8.parse::<usize>() else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, 
                                      "length must be an integer"));
        };
        let mut key = vec![0u8; inner_length + 2];
        reader.read_exact(&mut key).await?;
        if !key.ends_with(b"\r\n") {
            return Err(io::Error::new(io::ErrorKind::InvalidData, 
                                      "malformed element in array"));
        }
        key = key[..key.len()-2].to_vec();
        result.push(key);
    }
    Ok(Some(result))
}
