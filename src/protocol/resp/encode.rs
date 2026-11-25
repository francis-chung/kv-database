use crate::protocol::reply::Reply;

pub fn resp_encode(reply: &Reply, out: &mut Vec<u8>) {
    match reply {
        Reply::Ok => out.extend_from_slice(b"+OK\r\n"), 
        Reply::Nil => out.extend_from_slice(b"$-1\r\n"), 
        Reply::SimpleString(string) => out.extend_from_slice(&format!("+{string}\r\n").into_bytes()), 
        Reply::BulkString(vec) => {
            out.extend_from_slice(&format!("${}\r\n", vec.len()).into_bytes());
            out.extend_from_slice(vec);
            out.extend_from_slice(b"\r\n");
        }, 
        Reply::Integer(int) => out.extend_from_slice(&format!(":{int}\r\n").into_bytes()), 
        Reply::Array(vec) => {
            out.extend_from_slice(&format!("*{}\r\n", vec.len()).into_bytes());
            for item in vec {
                resp_encode(item, out);
            }
        }, 
        Reply::Error(e) => out.extend_from_slice(&format!("-{e}\r\n").into_bytes()),
    };
}