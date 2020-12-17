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
    pub fn put<K, V: serde::Serialize>(&mut self, key: K, value: &V)
    where
        K: DbWriteKey<V> + serde::Serialize,
    {
        let cf = K::cf(&self.db.cf);
        self.batch.put_cf(cf, serialize(&key), serialize(value));
    }

    #[inline(always)]
    pub fn delete<K, V>(&mut self, key: K)
    where
        K: DbWriteKey<V> + serde::Serialize,
    {
        let cf = K::cf(&self.db.cf);
        self.batch.delete_cf(cf, serialize(&key));
    }

    #[inline(always)]
    pub fn delete_range<K, V>(&mut self, from: K, to: K)
    where
        K: DbWriteKey<V> + serde::Serialize,
    {
        let cf = K::cf(&self.db.cf);
        self.batch
            .delete_range_cf(cf, serialize(&from), serialize(&to));
    }

    #[inline(always)]
    pub fn push<K, I: serde::Serialize>(&mut self, key: K, value: &I)
    where
        K: DbWriteKey<Vec<I>> + serde::Serialize,
    {
        let cf = K::cf(&self.db.cf);
        self.batch.merge_cf(cf, serialize(&key), serialize(value));
    }

    /// Perform the atomic batch write.
    #[inline(always)]
    pub fn write(self) -> Result<()> {
        self.db.db.write(self.batch).map_err(Error::DBError)
    }
}
