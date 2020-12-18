use super::DB;
use super::{bincode::serialize, keys::DbWriteKey};
use crate::error::{Error, Result};

/// An atomic batch of write operations. This is a type safe wrapper around
/// rocksdb::WriteBatch.
pub struct Batch<'a> {
    db: &'a DB,
    batch: rocksdb::WriteBatch,
}

impl<'a> Batch<'a> {
    pub fn new(db: &'a DB) -> Self {
        Batch {
            db,
            batch: rocksdb::WriteBatch::default(),
        }
    }

    #[inline(always)]
    pub fn put<K: DbWriteKey>(&mut self, key: K, value: &K::Value) {
        let cf = K::cf(&self.db.cf);
        self.batch
            .put_cf(cf, serialize(key.key()), serialize(value));
    }

    #[inline(always)]
    pub fn delete<K: DbWriteKey>(&mut self, key: K) {
        let cf = K::cf(&self.db.cf);
        self.batch.delete_cf(cf, serialize(key.key()));
    }

    #[inline(always)]
    pub fn delete_range<K: DbWriteKey>(&mut self, from: K, to: K) {
        let cf = K::cf(&self.db.cf);
        self.batch
            .delete_range_cf(cf, serialize(from.key()), serialize(to.key()));
    }

    #[inline(always)]
    pub fn push<K: DbWriteKey<Value = Vec<I>>, I: serde::Serialize>(&mut self, key: K, value: &I) {
        let cf = K::cf(&self.db.cf);
        self.batch
            .merge_cf(cf, serialize(key.key()), serialize(value));
    }

    /// Perform the atomic batch write.
    #[inline(always)]
    pub fn write(self) -> Result<()> {
        self.db.db.write(self.batch).map_err(Error::DBError)
    }
}
