use std::sync::Arc;
use tokio::sync::Mutex;

use crate::store::Db;
use crate::wal::WriteAheadLog;
use tokio::io::AsyncWrite;

pub struct Engine<W: AsyncWrite + Unpin> {
    pub store: Mutex<Db>, 
    pub logger: Mutex<WriteAheadLog<W>>
}

impl<W> Engine<W> 
where 
    W: AsyncWrite + Unpin
{
    pub fn new(store: Db, logger: WriteAheadLog<W>) -> Self {
        Self {
            store: Mutex::new(store), 
            logger: Mutex::new(logger),
        }
    }
}

pub(crate) type SharedEngine = Arc<Engine<tokio::fs::File>>;