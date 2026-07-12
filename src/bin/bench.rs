use std::{
    io::{Read, Write}, 
    net::TcpStream, 
    thread, 
    time::Instant
};

const ADDRESS: &str = "127.0.0.1:7878";
const THREADS: usize = 10;
const REQUESTS: usize = 100;
const REQUEST_TYPE: &str = "GET";

fn main() {
    let start = Instant::now();

    let handles: Vec<_> = (0..THREADS) 
        .map(|_| {
            thread::spawn(|| {
                let mut stream = TcpStream::connect(ADDRESS).unwrap();
                for i in 0..REQUESTS {
                    let request = request_string(REQUEST_TYPE, i, &REQUESTS);
                    stream.write_all(request.as_bytes()).unwrap();

                    let mut buf = [0u8; 1024];
                    let _bytes_read = stream.read(&mut buf).unwrap();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start.elapsed();
    let total_requests = THREADS * REQUESTS;
    let ops_per_sec = total_requests as f64 / elapsed.as_secs_f64();
    print!("total requests: {total_requests} in {elapsed:?} ({ops_per_sec:.4} ops/sec)");
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