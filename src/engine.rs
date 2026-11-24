use std::sync::Arc;
use std::sync::Mutex as SyncMutex;
use tokio::sync::Mutex as AsyncMutex;

use crate::store::Db;
use crate::wal::WriteAheadLog;
use tokio::io::AsyncWrite;

pub struct Engine<W: AsyncWrite + Unpin> {
    pub store: SyncMutex<Db>, 
    pub logger: AsyncMutex<WriteAheadLog<W>>
}

impl<W> Engine<W> 
where 
    W: AsyncWrite + Unpin
{
    pub fn new(store: Db, logger: WriteAheadLog<W>) -> Self {
        Self {
            store: SyncMutex::new(store), 
            logger: AsyncMutex::new(logger),
        }
    }
}

pub(crate) type SharedEngine = Arc<Engine<tokio::fs::File>>;
