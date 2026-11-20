use std::io;
use tokio::{
    io::{AsyncBufReadExt, BufReader}, 
    net::tcp::OwnedReadHalf
};

pub async fn line_decode(
    reader: &mut BufReader<OwnedReadHalf>, 
    line: &mut Vec<u8>
) -> io::Result<usize> {
    line.clear();
    reader.read_until(b'\n', line).await
}