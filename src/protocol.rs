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

// implements basic get / set / del operations
pub fn parse_command(line_bytes: &[u8]) -> Result<Command, ProtocolError> {
    let line = str::from_utf8(line_bytes).map_err(|_| ProtocolError::InvalidUtf8)?;
    let mut words = line.split_whitespace();
    match words.next() {
        None => Err(ProtocolError::Empty),
        Some(cmd) => match cmd.to_ascii_uppercase().as_str() {
            "GET" => {
                // ok_or converts an Option into a Result, which returns
                // early if an error occurred
                let key = words.next().ok_or(ProtocolError::WrongArity)?;
                if words.next().is_some() {
                    return Err(ProtocolError::WrongArity);
                }
                Ok(Command::Get {
                    key: key.to_string(),
                })
            }
            "SET" => {
                let key = words.next().ok_or(ProtocolError::WrongArity)?;
                let value = words.next().ok_or(ProtocolError::WrongArity)?;
                if words.next().is_some() {
                    return Err(ProtocolError::WrongArity);
                }
                Ok(Command::Set {
                    key: key.to_string(),
                    value: value.to_string(),
                })
            }
            "DEL" => {
                let key = words.next().ok_or(ProtocolError::WrongArity)?;
                if words.next().is_some() {
                    return Err(ProtocolError::WrongArity);
                }
                Ok(Command::Del {
                    key: key.to_string(),
                })
            }
            "EXISTS" => {
                let key = words.next().ok_or(ProtocolError::WrongArity)?;
                if words.next().is_some() {
                    return Err(ProtocolError::WrongArity);
                }
                Ok(Command::Exists { 
                    key: key.to_string()
                })
            }
            "DBSIZE" => {
                if words.next().is_some() {
                    return Err(ProtocolError::WrongArity);
                }
                Ok(Command::DbSize)
            }
            "CLEAR" => {
                if words.next().is_some() {
                    return Err(ProtocolError::WrongArity);
                }
                Ok(Command::Clear)
            }
            "ZADD" => {
                let key = words.next().ok_or(ProtocolError::WrongArity)?;
                let member = words.next().ok_or(ProtocolError::WrongArity)?;
                let score_text = words.next().ok_or(ProtocolError::WrongArity)?;
                if words.next().is_some() {
                    return Err(ProtocolError::WrongArity);
                }
                let score = score_text.parse::<f64>();
                if let Err(_) = score {
                    return Err(ProtocolError::WrongType("score".to_string()));
                }
                Ok(Command::Zadd {
                    key: key.to_string(), 
                    member: member.to_string(), 
                    score: OrderedFloat(score.unwrap())
                })
            }
            "ZSCORE" => {
                let key = words.next().ok_or(ProtocolError::WrongArity)?;
                let member = words.next().ok_or(ProtocolError::WrongArity)?;
                if words.next().is_some() {
                    return Err(ProtocolError::WrongArity);
                }
                Ok(Command::Zscore {
                    key: key.to_string(), 
                    member: member.to_string()
                })
            }
            "ZREM" => {
                let key = words.next().ok_or(ProtocolError::WrongArity)?;
                let member = words.next().ok_or(ProtocolError::WrongArity)?;
                if words.next().is_some() {
                    return Err(ProtocolError::WrongArity);
                }
                Ok(Command::Zrem {
                    key: key.to_string(), 
                    member: member.to_string()
                })
            }
            "ZRANGE" => {
                let key = words.next().ok_or(ProtocolError::WrongArity)?;
                let from_text = words.next().ok_or(ProtocolError::WrongArity)?;
                let to_text = words.next().ok_or(ProtocolError::WrongArity)?;
                let with_scores_opt = words.next();
                if words.next().is_some() {
                    return Err(ProtocolError::WrongArity);
                }
                let from = from_text.parse::<isize>();
                if let Err(_) = from {
                    return Err(ProtocolError::WrongType("from".to_string()));
                }
                let to = to_text.parse::<isize>();
                if let Err(_) = to {
                    return Err(ProtocolError::WrongType("to".to_string()));
                }
                let mut with_scores_val: bool;
                if let Some(with_scores) = with_scores_opt {
                    let converted = with_scores.to_ascii_uppercase();
                    if converted.as_str() == "WITHSCORES" {
                        with_scores_val = true;
                    } else {
                        return Err(ProtocolError::UnknownKeyword(converted));
                    }
                } else {
                    with_scores_val = false;
                }
                Ok(Command::Zrange {
                    key: key.to_string(), 
                    from: from.unwrap(), 
                    to: to.unwrap(), 
                    with_scores: with_scores_val
                })
            }
            other => Err(ProtocolError::UnknownCommand(other.to_string())),
        },
    }
}
