use std::io;

use crate::protocol::reply::Reply;

pub fn resp_encode(reply: Reply) -> io::Result<String> {
    match reply {
        Reply::Ok => Ok("+OK\r\n".to_string()), 
        Reply::Nil => Ok("$-1\r\n".to_string()), 
        Reply::SimpleString(string) => Ok(format!("+{string}\r\n")), 
        Reply::BulkString(vec) => {
            let str = String::from_utf8_lossy_owned(vec);
            Ok(format!("${}\r\n{str}\r\n", str.chars().count()))
        }, 
        Reply::Integer(int) => Ok(format!(":{int}\r\n")), 
        Reply::Array(vec) => {
            let len = vec.len();
            let mut response = format!("*{len}\r\n");
            for rep in vec.iter() {
                if let Reply::BulkString(inner_vec) = rep {
                    let str = String::from_utf8_lossy_owned(inner_vec.clone());
                    response.push_str(format!("${}\r\n{str}\r\n", str.len()).as_str());
                }
            }
            Ok(response)
        }, 
        Reply::Error(e) => Ok(format!("-{e}\r\n"))
    }
}