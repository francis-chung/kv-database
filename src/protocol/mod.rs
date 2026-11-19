use ordered_float::OrderedFloat;

pub enum Command {
    Get { key: String },
    Set { key: String, value: String },
    Del { key: String },
    Exists { key: String }, 
    DbSize, 
    Clear, 
    Zadd { key: String, member: String, score: OrderedFloat<f64> }, 
    Zscore { key: String, member: String }, 
    Zrem { key: String, member: String }, 
    Zrange { key: String, from: isize, to: isize, with_scores: bool }
}

#[derive(Debug)]
pub enum ProtocolError {
    Empty,
    UnknownCommand(String),
    UnknownKeyword(String),
    WrongArity,
    WrongType(String),
    InvalidUtf8,
}

pub mod command;
pub mod reply;
pub mod line;