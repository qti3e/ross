use super::keys::DBKey;
use super::DB;
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
        K: DBKey<V>,
    {
        let key = key.serialize();
        let cf = K::cf(&self.db.cf);
        let value = bincode::serialize(value).unwrap();
        self.batch.put_cf(cf, key, value);
    }

    #[inline(always)]
    pub fn delete<K, V>(&mut self, key: K)
    where
        K: DBKey<V>,
    {
        let key = key.serialize();
        let cf = K::cf(&self.db.cf);
        self.batch.delete_cf(cf, key);
    }

    #[inline(always)]
    pub fn delete_range<K, V>(&mut self, from: K, to: K)
    where
        K: DBKey<V>,
    {
        let from = from.serialize();
        let to = to.serialize();
        let cf = K::cf(&self.db.cf);
        self.batch.delete_range_cf(cf, from, to);
    }

    #[inline(always)]
    pub fn push<K, I: serde::Serialize>(&mut self, key: K, value: &I)
    where
        K: DBKey<Vec<I>>,
    {
        let key = key.serialize();
        let value = bincode::serialize(value).unwrap();
        let cf = K::cf(&self.db.cf);
        self.batch.merge_cf(cf, key, value);
    }

    /// Perform the atomic batch write.
    #[inline(always)]
    pub fn write(self) -> Result<()> {
        self.db.db.write(self.batch).map_err(|e| Error::DBError(e))
    }
}
