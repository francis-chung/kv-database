use std::{
    fs, 
    io::{BufReader, prelude::*}, 
    net::{TcpListener, TcpStream}, 
    thread, 
    time::Duration
};

use kv_database::ThreadPool;

// begins watching the address and delegating connection handling
pub fn start_connection() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        // relies on thread pool to opreate any tasks
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

// returns response based on request
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // only evaluates first line of HTTP request
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"), 
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }, 
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    // formats everything in HTTP response format
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // sends response in byte form back down connection
    stream.write_all(response.as_bytes()).unwrap();
}