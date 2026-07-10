pub enum Command {
    Get { key: String }, 
    Set { key: String, value: String }, 
    Del { key: String }
}

#[derive(Debug)]
pub enum ProtocolError {
    Empty, 
    UnknownCommand(String), 
    WrongArity
}

// implements basic get / set / del operations
pub fn parse_command(line: &str) -> Result<Command, ProtocolError> {
    let mut words = line.split_whitespace();
    match words.next() {
        None => Err(ProtocolError::Empty), 
        Some(cmd) => match cmd.to_ascii_uppercase().as_str() {
            "GET" => { 
                // ok_or converts an Option into a Result, which returns 
                // early if an error occurred
                let key = words.next().ok_or(ProtocolError::WrongArity)?;
                Ok(Command::Get { key: key.to_string() })
            }
            "SET" => {
                let key = words.next().ok_or(ProtocolError::WrongArity)?;
                let value = words.next().ok_or(ProtocolError::WrongArity)?;
                Ok(Command::Set { key: key.to_string(), value: value.to_string() })
            }
            "DEL" => {
                let key = words.next().ok_or(ProtocolError::WrongArity)?;
                Ok(Command::Del { key: key.to_string() })
            }
            other => Err(ProtocolError::UnknownCommand(other.to_string()))
        }
    }
}
