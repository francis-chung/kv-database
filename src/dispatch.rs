use crate::engine::SharedEngine;
use crate::wal::encode_record;
use kv_database::protocol::{
    Command, 
    reply::Reply,
};

pub(crate) async fn dispatch(cmd: Command, engine: SharedEngine) -> Reply {
    match cmd {
        Command::Pong => Reply::SimpleString("PONG".to_string()),
        Command::Get { key } => {
            let mut store = engine.store.lock().await;
            match store.kv_store.get(&key) {
                Some(value) => Reply::BulkString(value.into_bytes()), 
                None => Reply::Nil
            }
        }
        Command::Set { key, value } => {
            let bytes = encode_record(&Command::Set { key: key.clone(), value: value.clone() });
            if let Err(reply) = log(&engine, &bytes).await {
                return reply;
            }
            engine.store.lock().await.kv_store.insert(key, value);
            Reply::Ok
        }
        Command::Del { key } => {
            let bytes = encode_record(&Command::Del { key: key.clone() });
            if let Err(reply) = log(&engine, &bytes).await {
                return reply;
            }
            engine.store.lock().await.kv_store.remove(&key);
            Reply::Ok
        }
        Command::Exists { key } => {
            let mut store = engine.store.lock().await;
            match store.kv_store.contains_key(&key) {
                true => Reply::Integer(1),
                false => Reply::Integer(0)
            }
        }
        Command::DbSize => {
            let result = engine.store.lock().await.kv_store.len();
            Reply::Integer(result)
        }
        Command::Clear => {
            engine.store.lock().await.kv_store.clear();
            Reply::Ok
        }
        Command::Zadd { key, member, score } => {
            let bytes = encode_record(&Command::Zadd { key: key.clone(), member: member.clone(), score });
            if let Err(reply) = log(&engine, &bytes).await {
                return reply;
            }
            engine.store.lock().await.sorted_sets.zadd(&key, member, score);
            Reply::Ok
        }
        Command::Zscore { key, member } => {
            let store = engine.store.lock().await;
            match store.sorted_sets.zscore(&key, &member) {
                Some(value) => Reply::BulkString(value.to_string().into_bytes()),
                None => Reply::Nil
            }
        }
        Command::Zrem { key, member } => {
            let bytes = encode_record(&Command::Zrem { key: key.clone(), member: member.clone() });
            if let Err(reply) = log(&engine, &bytes).await {
                return reply;
            }
            engine.store.lock().await.sorted_sets.zrem(&key, &member);
            Reply::Ok
        }
        Command::Zrange { key, from, to, with_scores } => {
            let store = engine.store.lock().await;
            match store.sorted_sets.zrange(&key, from, to, with_scores) {
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

async fn log(engine: &SharedEngine, bytes: &Vec<u8>) -> Result<(), Reply> {
    let mut logger = engine.logger.lock().await;
    if let Err(e) = logger.buffered_log(&bytes).await {
        return Err(Reply::Error(format!("ERR {e}")));
    }
    Ok(())
}