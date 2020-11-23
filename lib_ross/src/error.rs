use rocksdb;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    DBError(rocksdb::Error),
    AcquireWriteLock,
    AcquireReadLock,
    AcquireLock,
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::DBError(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DBError(e) => write!(f, "RocksDB error: {}", e),
            Error::AcquireWriteLock => write!(f, "Could not acquire the write lock to DB."),
            Error::AcquireReadLock => write!(f, "Could not acquire the read lock to DB."),
            Error::AcquireLock => write!(f, "Could not acquire a lock."),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
