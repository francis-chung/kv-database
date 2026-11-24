use crate::protocol::reply::Reply;

pub fn resp_encode(reply: &Reply) -> Vec<u8> {
    match reply {
        Reply::Ok => b"+OK\r\n".to_vec(), 
        Reply::Nil => b"$-1\r\n".to_vec(), 
        Reply::SimpleString(string) => format!("+{string}\r\n").into_bytes(), 
        Reply::BulkString(vec) => {
            let mut out = format!("${}\r\n", vec.len()).into_bytes();
            out.extend_from_slice(vec);
            out.extend_from_slice(b"\r\n");
            out
        }, 
        Reply::Integer(int) => format!(":{int}\r\n").into_bytes(), 
        Reply::Array(vec) => {
            let mut out = format!("*{}\r\n", vec.len()).into_bytes();
            for item in vec {
                out.extend(resp_encode(item));
            }
            out
        }, 
        Reply::Error(e) => format!("-{e}\r\n").into_bytes()
    }
}