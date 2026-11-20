use ordered_float::OrderedFloat;

use crate::protocol::{
    Command, 
    ProtocolError,
};

pub fn parse_tokens(tokens: &[Vec<u8>]) -> Result<Command, ProtocolError> {
    let len = tokens.len();
    if len == 0 {
        return Err(ProtocolError::Empty);
    }
    match convert_utf8(&tokens[0])?.to_ascii_uppercase().as_str() {
        "GET" => {
            arity(tokens, 2)?;
            let key = convert_utf8(&tokens[1])?;
            Ok(Command::Get { key })
        }, 
        "SET" => {
            arity(tokens, 3)?;
            let key = convert_utf8(&tokens[1])?;
            let value = convert_utf8(&tokens[2])?;
            Ok(Command::Set { key, value })
        }, 
        "DEL" => {
            arity(tokens, 2)?;
            let key = convert_utf8(&tokens[1])?;
            Ok(Command::Del { key })
        }, 
        "EXISTS" => {
            arity(tokens, 2)?;
            let key = convert_utf8(&tokens[1])?;
            Ok(Command::Exists { key })
        },
        "DBSIZE" => {
            arity(tokens, 1)?;
            Ok(Command::DbSize)
        },
        "CLEAR" => {
            arity(tokens, 1)?;
            Ok(Command::Clear)
        },
        "ZADD" => {
            arity(tokens, 4)?;
            let key = convert_utf8(&tokens[1])?;
            let member = convert_utf8(&tokens[2])?;
            let score_text = convert_utf8(&tokens[3])?;
            let score = OrderedFloat(score_text.parse::<f64>()
                                    .map_err(|_| ProtocolError::WrongType("score".to_string()))?);
            Ok(Command::Zadd { key, member, score })
        }, 
        "ZSCORE" => {
            arity(tokens, 3)?;
            let key = convert_utf8(&tokens[1])?;
            let member = convert_utf8(&tokens[2])?;
            Ok(Command::Zscore { key, member })
        }, 
        "ZREM" => {
            arity(tokens, 3)?;
            let key = convert_utf8(&tokens[1])?;
            let member = convert_utf8(&tokens[2])?;
            Ok(Command::Zrem { key, member })
        }, 
        "ZRANGE" => {
            if len != 4 && len != 5 {
                return Err(ProtocolError::WrongArity);
            }
            let key = convert_utf8(&tokens[1])?;
            let from_text = convert_utf8(&tokens[2])?;
            let from = from_text.parse::<isize>()
                                .map_err(|_| ProtocolError::WrongType("from".to_string()))?;
            let to_text = convert_utf8(&tokens[3])?;
            let to = to_text.parse::<isize>()
                            .map_err(|_| ProtocolError::WrongType("to".to_string()))?;
            let mut with_scores = false;
            if len == 5 {
                let with_scores_text = convert_utf8(&tokens[4])?;
                if with_scores_text.to_ascii_uppercase().as_str() == "WITHSCORES" {
                    with_scores = true;
                } else {
                    return Err(ProtocolError::UnknownKeyword(with_scores_text))
                }
            }
            Ok(Command::Zrange{ key, from, to, with_scores })
        }, 
        other => Err(ProtocolError::UnknownCommand(other.to_string()))
    }
}

fn arity(tokens: &[Vec<u8>], len: usize) -> Result<(), ProtocolError> {
    if tokens.len() != len {
        return Err(ProtocolError::WrongArity);
    }
    Ok(())
}

fn convert_utf8(value: &[u8]) -> Result<String, ProtocolError> {
    String::from_utf8(value.to_vec()).map_err(|_| ProtocolError::InvalidUtf8)
}

pub fn parse_command(line_bytes: &[u8]) -> Result<Command, ProtocolError> {
    let tokens: Vec<Vec<u8>> = line_bytes
        .split(|b| b.is_ascii_whitespace())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_vec())
        .collect();
    parse_tokens(&tokens)
}
