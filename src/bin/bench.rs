use std::{
    time::Instant
};
use tokio::{
    net::TcpStream, 
    io::{AsyncWriteExt, AsyncReadExt}
};


const ADDRESS: &str = "127.0.0.1:7878";
const THREADS: usize = 1000;
const REQUESTS: usize = 100;
const REQUEST_TYPE: &str = "GET/SET";

#[tokio::main]
async fn main() {
    let start = Instant::now();

    let handles: Vec<_> = (0..THREADS) 
        .map(|_| {
            tokio::spawn(async move {
                let mut stream = TcpStream::connect(ADDRESS).await?;
                for i in 0..REQUESTS {
                    let request = request_string(REQUEST_TYPE, i, &REQUESTS);
                    stream.write_all(request.as_bytes()).await?;

                    let mut buf = [0u8; 1024];
                    let _bytes_read = stream.read(&mut buf).await?;
                }
                Ok::<(), tokio::io::Error>(())
            })
        })
        .collect();

    for handle in handles {
        match handle.await {
            Ok(_) => {}
            Err(e) => eprintln!("Task panicked or was cancelled: {e:?}\n") 
        }
    }

    let elapsed = start.elapsed();
    let total_requests = THREADS * REQUESTS;
    let ops_per_sec = total_requests as f64 / elapsed.as_secs_f64();
    println!("total requests: {total_requests} in {elapsed:?} ({ops_per_sec:.4} ops/sec)");
}

fn request_string(format: &str, num: usize, total: &usize) -> String {
    match format {
        "GET" => format!("GET key{num}\n"), 
        "SET" => format!("SET key{num} value{num}\n"), 
        "GET/SET" => {
            match num {
                x if x < total / 2 => format!("GET key{x}\n"), 
                x => format!("SET key{} value{}\n", total / 2 - x, total / 2 - x)
            }
        }
        _ => format!("ERROR\n")
    }
}