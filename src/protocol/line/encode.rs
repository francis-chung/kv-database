use std::io;

use crate::protocol::reply::Reply;

pub fn line_encode(reply: Reply) -> io::Result<String> {
    match reply {
        Reply::Ok => Ok("OK\n".to_string()), 
        Reply::Nil => Ok("NIL\n".to_string()), 
        Reply::SimpleString(string) => Ok(format!("{string}\n")),
        Reply::BulkString(vec) => {
            let str = String::from_utf8_lossy_owned(vec);
            Ok(format!("{str}\n"))
        }
        Reply::Integer(int) => Ok(format!("{}\n", int)), 
        Reply::Array(vec) => {
            let mut response = String::new();
            for (index, rep) in vec.iter().enumerate() {
                if let Reply::BulkString(value) = rep {
                    response += &format!("{}) \"{}\"\n", index + 1, 
                                          String::from_utf8_lossy_owned(value.clone()));
                }
            }
            Ok(response)
        } 
        Reply::Error(e) => Ok(format!("ERR {e}\n"))
    }
}
