use std::{
    io, 
    sync::Arc,
};
use tokio::sync::Mutex;

use crate::engine::Engine;
use crate::protocol::Command;
use crate::wal::encode_record;

pub(crate) type MutexEngine = Arc<Mutex<Engine<tokio::fs::File>>>;

pub(crate) async fn dispatch(cmd: Command, engine: MutexEngine) -> io::Result<String> {
    match cmd {
        Command::Get { key } => {
            let mut eng = engine.lock().await;
            match eng.store.kv_store.get(&key) {
                Some(value) => Ok(format!("VALUE {value}\n")),
                None => Ok("NIL\n".to_string())
            }
        }
        Command::Set { key, value } => {
            let mut eng = engine.lock().await;
            let bytes = encode_record(&Command::Set { key: key.clone(), value: value.clone() });
            eng.logger.buffered_log(&bytes).await?;
            eng.store.kv_store.insert(key, value);
            Ok("OK\n".to_string())
        }
        Command::Del { key } => {
            let mut eng = engine.lock().await;
            let bytes = encode_record(&Command::Del { key: key.clone() });
            eng.logger.buffered_log(&bytes).await?;
            eng.store.kv_store.remove(&key);
            Ok("OK\n".to_string())
        }
        Command::Exists { key } => {
            let mut eng = engine.lock().await;
            match eng.store.kv_store.contains_key(&key) {
                true => Ok("1\n".to_string()),
                false => Ok("0\n".to_string())
            }
        }
        Command::DbSize => {
            let result = engine.lock().await.store.kv_store.len();
            Ok(format!("{result}\n"))
        }
        Command::Clear => {
            engine.lock().await.store.kv_store.clear();
            Ok("OK\n".to_string())
        }
        Command::Zadd { key, member, score } => {
            let mut eng = engine.lock().await;
            let bytes = encode_record(&Command::Zadd { key: key.clone(), member: member.clone(), score });
            eng.logger.buffered_log(&bytes).await?;
            eng.store.sorted_sets.zadd(&key, member, score);
            Ok("OK\n".to_string())
        }
        Command::Zscore { key, member } => {
            let eng = engine.lock().await;
            match eng.store.sorted_sets.zscore(&key, &member) {
                Some(value) => Ok(format!("VALUE {value}\n")),
                None => Ok("NIL\n".to_string())
            }
        }
        Command::Zrem { key, member } => {
            let mut eng = engine.lock().await;
            let bytes = encode_record(&Command::Zrem { key: key.clone(), member: member.clone() });
            eng.logger.buffered_log(&bytes).await?;
            eng.store.sorted_sets.zrem(&key, &member);
            Ok("OK\n".to_string())
        }
        Command::Zrange { key, from, to, with_scores } => {
            let mut response = String::new();
            let eng = engine.lock().await;
            if let Some(rows) = eng.store.sorted_sets.zrange(&key, from, to, with_scores) {
                for (index, (key, poss_value)) in rows.iter().enumerate() {
                    if let Some(val) = poss_value {
                        response += &format!("{}) \"{key}\"\n", 2 * index + 1);
                        response += &format!("{}) \"{val}\"\n", 2 * index + 2);
                    } else {
                        response += &format!("{}) \"{key}\"\n", index + 1);
                    }
                }
            } else {
                response = "NIL\n".to_string();
            }
            Ok(response)
        }
    }
}
