use super::keys::DBKey;

/// An atomic batch of write operations. This is a type safe wrapper around
/// rocksdb::WriteBatch.
pub struct Batch {
    batch: rocksdb::WriteBatch,
}

impl Batch {
    pub fn new() -> Self {
        Batch {
            batch: rocksdb::WriteBatch::default(),
        }
    }

    #[inline(always)]
    pub fn put<K, V: serde::Serialize>(&mut self, key: K, value: &V)
    where
        K: DBKey<V>,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        let value = bincode::serialize(value).unwrap();
        self.batch.put(key, value);
    }

    #[inline(always)]
    pub fn delete<K, V>(&mut self, key: K)
    where
        K: DBKey<V>,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        self.batch.delete(key);
    }

    #[inline(always)]
    pub fn delete_range<K, V>(&mut self, from: K, to: K)
    where
        K: DBKey<V>,
    {
        let from = bincode::serialize(&from.key()).unwrap();
        let to = bincode::serialize(&to.key()).unwrap();
        self.batch.delete_range(from, to);
    }

    #[inline(always)]
    pub fn push<K, I: serde::Serialize>(&mut self, key: K, item: &I)
    where
        K: DBKey<Vec<I>>,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        let item = bincode::serialize(item).unwrap();
        self.batch.merge(key, item);
    }

    #[inline(always)]
    pub(super) fn finalize(self) -> rocksdb::WriteBatch {
        self.batch
    }
}
