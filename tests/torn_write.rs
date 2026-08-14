use std::{io, pin::Pin, task::{Context, Poll}};
use tokio::io::AsyncWrite;

use kv_database::wal::{WriteAheadLog, encode_record, replay_from_bytes};
use kv_database::protocol::Command;
use kv_database::store::Db;

struct TornWriter {
    buf: Vec<u8>, 
    limit: usize, 
}

impl AsyncWrite for TornWriter {
    // base func: sends bytes to destination, possibly buffering
    fn poll_write(
        mut self: Pin<&mut Self>, 
        _cx: &mut Context<'_>, 
        data: &[u8],
    ) -> Poll<io::Result<usize>> {
        // only writes to buffer up to a certain limit of characters
        let remaining = self.limit.saturating_sub(self.buf.len());
        let n = data.len().min(remaining);
        self.buf.extend_from_slice(&data[..n]);
        Poll::Ready(Ok(n))
    }

    // base func: force buffered bytes to go to destination
    // not required because bytes enter destination immediately
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    // base func: cleanly close writer, flushing remaining data and closing resource
    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn torn_write() {
    let record1 = encode_record(&Command::Set { key: "k1".to_string(), value: "v1".to_string() });
    let record2 = encode_record(&Command::Set { key: "k2".to_string(), value: "v2".to_string() });

    let limit = record1.len() + record2.len() / 2;
    let torn = TornWriter { buf: Vec::new(), limit };
    let Ok(mut wal) = WriteAheadLog::new(torn).await else {
        eprintln!("Failed to initialize logger");
        return;
    };
    
    let _ = wal.buffered_log(&record1).await;
    let _ = wal.buffered_log(&record2).await;

    let bytes = wal.writer.get_ref().buf.clone();
    let mut db = Db::new();
    let good_len = replay_from_bytes(&bytes, &mut db).unwrap();

    assert_eq!(good_len, record1.len() as u64);
    println!("Torn write test passed: good_len = {good_len}");
}