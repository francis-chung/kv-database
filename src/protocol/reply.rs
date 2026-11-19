pub enum Reply {
    Ok, 
    Nil, 
    SimpleString(String), 
    BulkString(Vec<u8>), 
    Integer(usize), 
    Array(Vec<Reply>), 
    Error(String),
}