use std::io;

use crate::engine::MutexEngine;
use crate::wal::encode_record;
use kv_database::protocol::{
    Command, 
    reply::Reply,
};

pub(crate) async fn dispatch(cmd: Command, engine: MutexEngine) -> Reply {
    match cmd {
        Command::Get { key } => {
            let mut eng = engine.lock().await;
            match eng.store.kv_store.get(&key) {
                Some(value) => Reply::BulkString(value.into_bytes()), 
                None => Reply::Nil
            }
        }
        Command::Set { key, value } => {
            let mut eng = engine.lock().await;
            let bytes = encode_record(&Command::Set { key: key.clone(), value: value.clone() });
            if let Err(e) = eng.logger.buffered_log(&bytes).await {
                return Reply::Error(e.to_string());
            }
            eng.store.kv_store.insert(key, value);
            Reply::Ok
        }
        Command::Del { key } => {
            let mut eng = engine.lock().await;
            let bytes = encode_record(&Command::Del { key: key.clone() });
            if let Err(e) = eng.logger.buffered_log(&bytes).await {
                return Reply::Error(e.to_string());
            }
            eng.store.kv_store.remove(&key);
            Reply::Ok
        }
        Command::Exists { key } => {
            let mut eng = engine.lock().await;
            match eng.store.kv_store.contains_key(&key) {
                true => Reply::Integer(1),
                false => Reply::Integer(0)
            }
        }
        Command::DbSize => {
            let result = engine.lock().await.store.kv_store.len();
            Reply::Integer(result)
        }
        Command::Clear => {
            engine.lock().await.store.kv_store.clear();
            Reply::Ok
        }
        Command::Zadd { key, member, score } => {
            let mut eng = engine.lock().await;
            let bytes = encode_record(&Command::Zadd { key: key.clone(), member: member.clone(), score });
            if let Err(e) = eng.logger.buffered_log(&bytes).await {
                return Reply::Error(e.to_string());
            }
            eng.store.sorted_sets.zadd(&key, member, score);
            Reply::Ok
        }
        Command::Zscore { key, member } => {
            let eng = engine.lock().await;
            match eng.store.sorted_sets.zscore(&key, &member) {
                Some(value) => Reply::BulkString(value.to_string().into_bytes()),
                None => Reply::Nil
            }
        }
        Command::Zrem { key, member } => {
            let mut eng = engine.lock().await;
            let bytes = encode_record(&Command::Zrem { key: key.clone(), member: member.clone() });
            if let Err(e) = eng.logger.buffered_log(&bytes).await {
                return Reply::Error(e.to_string());
            }
            eng.store.sorted_sets.zrem(&key, &member);
            Reply::Ok
        }
        Command::Zrange { key, from, to, with_scores } => {
            let eng = engine.lock().await;
            match eng.store.sorted_sets.zrange(&key, from, to, with_scores) {
                Some (rows) => {
                    let mut items = Vec::new();
                    for (member, score) in rows {
                        items.push(Reply::BulkString(member.into_bytes()));
                        if let Some(score) = score {
                            items.push(Reply::BulkString(score.to_string().into_bytes()));
                        }
                    }
                    Reply::Array(items)
                }
                None => Reply::Nil,
            }
        }
    }
}
