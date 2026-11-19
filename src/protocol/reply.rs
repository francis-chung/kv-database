pub enum Reply {
    Ok, 
    SimpleString(String), 
    BulkString(Vec<u8>), 
    Nil, 
    Integer(usize), 
    Array(Vec<Reply>), 
    Error(String),
}