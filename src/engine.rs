use crate::store::Db;
use crate::wal::WriteAheadLog;
use tokio::io::AsyncWrite;

pub struct Engine<W: AsyncWrite + Unpin> {
    pub store: Db, 
    pub logger: WriteAheadLog<W>
}