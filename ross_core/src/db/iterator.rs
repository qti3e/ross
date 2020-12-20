use super::bincode::deserialize;
use std::marker::PhantomData;

/// An iterator that iterates over a range of keys in the DB.
pub struct KeyIterator<'a, K> {
    pub(super) inner: rocksdb::DBIterator<'a>,
    pub(super) phantom: PhantomData<K>,
}

impl<'a, K> Iterator for KeyIterator<'a, K>
where
    K: serde::de::DeserializeOwned,
{
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, _)| deserialize(k.as_ref()))
    }
}

/// An iterator that iterates over a range of key-value pairs in the DB.
pub struct KeyValueIterator<'a, K, V> {
    pub(super) inner: rocksdb::DBIterator<'a>,
    pub(super) phantom: PhantomData<(K, V)>,
}

impl<'a, K, V> Iterator for KeyValueIterator<'a, K, V>
where
    K: serde::de::DeserializeOwned,
    V: serde::de::DeserializeOwned,
{
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(k, v)| (deserialize(k.as_ref()), deserialize(v.as_ref())))
    }
}
