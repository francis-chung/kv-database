use crate::store::Db;
use crate::wal::WriteAheadLog;

pub struct Engine {
    pub store: Db, 
    pub logger: WriteAheadLog
}