use std::io;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

const MAX_ARRAY_LEN: usize = 1024;
const MAX_BULK_LEN: usize = 512 * 1024 * 1024;

pub async fn resp_decode<R>(reader: &mut R) -> io::Result<Option<Vec<Vec<u8>>>>
where 
    R: AsyncBufRead + Unpin,
{
    let mut line = Vec::new();

    // header: *<count>\r\n
    if reader.read_until(b'\n', &mut line).await? == 0 {
        return Ok(None)
    }
    let header = strip_crlf(&line)?;
    let count = parse_prefixed(header, b'*', MAX_ARRAY_LEN)?;

    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        line.clear();
        
        // bulk string: $<len>\r\n
        if reader.read_until(b'\n', &mut line).await? == 0 {
            return Err(eof("unexpected EOF reading bulk header"));
        }
        let bulk_header = strip_crlf(&line)?;
        let len = parse_prefixed(bulk_header, b'$', MAX_BULK_LEN)?;

        let mut buf = vec![0u8; len + 2];
        reader.read_exact(&mut buf).await?;
        if !buf.ends_with(b"\r\n") {
            return Err(invalid("bulk string not terminated by CRLF"));
        }
        buf.truncate(len);
        result.push(buf);
    }
    Ok(Some(result))
}

fn strip_crlf(line: &[u8]) -> io::Result<&[u8]> {
    line.strip_suffix(b"\r\n").ok_or_else(|| invalid("line not terminated by CRLF"))
}

fn parse_prefixed(line: &[u8], prefix: u8, max: usize) -> io::Result<usize> {
    if line.first() != Some(&prefix) {
        return Err(invalid("wrong type prefix"));
    }
    let n = str::from_utf8(&line[1..])
        .map_err(|_| invalid("length not UTF-8"))?
        .parse()
        .map_err(|_| invalid("length not integer"))?;
    if n > max {
        return Err(invalid("length exceeds max"));
    }
    Ok(n)
}

fn invalid(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

fn eof(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::UnexpectedEof, msg)
}