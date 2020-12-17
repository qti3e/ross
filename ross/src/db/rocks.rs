use super::{
    keys::{self, DBKey, ReadOnlyDBKey, CF},
    Batch,
};
use crate::error::{Error, Result};
use crate::utils::bincode_vec_push::merge_push;
use std::marker::PhantomData;

/// A type safe wrapper around rocksdb.
pub struct DB {
    pub(super) db: rocksdb::DB,
    pub(super) cf: CF,
}

impl DB {
    /// Open a new database instance.
    pub fn open(path: &str) -> Self {
        // TODO(qti3e) Support options.
        let mut options = rocksdb::Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        let db = rocksdb::DB::open_cf_descriptors(
            &options,
            path,
            vec![
                rocksdb::ColumnFamilyDescriptor::new(keys::REPOSITORIES, {
                    rocksdb::Options::default()
                }),
                rocksdb::ColumnFamilyDescriptor::new(keys::BRANCHES, {
                    rocksdb::Options::default()
                }),
                rocksdb::ColumnFamilyDescriptor::new(keys::MERGE_BRANCHES, {
                    rocksdb::Options::default()
                }),
                rocksdb::ColumnFamilyDescriptor::new(keys::COMMITS, {
                    rocksdb::Options::default()
                }),
                rocksdb::ColumnFamilyDescriptor::new(keys::SNAPSHOT, {
                    rocksdb::Options::default()
                }),
                rocksdb::ColumnFamilyDescriptor::new(keys::LOG, {
                    let mut opts = rocksdb::Options::default();
                    opts.set_merge_operator("bincode_push", vec_push_merge, None);
                    opts
                }),
                rocksdb::ColumnFamilyDescriptor::new(keys::LIVE_CHANGES, {
                    let mut opts = rocksdb::Options::default();
                    opts.set_merge_operator("bincode_push", vec_push_merge, None);
                    opts
                }),
                rocksdb::ColumnFamilyDescriptor::new(keys::PACKED_DELTA, {
                    rocksdb::Options::default()
                }),
            ],
        )
        .unwrap();
        let cf = CF::new(&db);

        DB { db, cf }
    }

    /// Create a new batch.
    #[inline]
    pub fn batch<'a>(&'a self) -> Batch<'a> {
        Batch::new(self)
    }

    pub fn get<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: ReadOnlyDBKey<V>,
        V: serde::de::DeserializeOwned,
    {
        let key = key.serialize();
        let cf = K::cf(&self.cf);
        let pinned = self.db.get_pinned_cf(cf, key).map_err(Error::DBError)?;
        let bytes = match pinned {
            Some(slice) => slice,
            None => return Ok(None),
        };
        let data = bincode::deserialize(bytes.as_ref()).unwrap();
        Ok(Some(data))
    }

    #[inline(always)]
    pub fn push<K, I: serde::Serialize>(&self, key: K, value: &I) -> Result<()>
    where
        K: DBKey<Vec<I>>,
    {
        let key = key.serialize();
        let cf = K::cf(&self.cf);
        let value = bincode::serialize(value).unwrap();
        self.db.merge_cf(cf, key, value).map_err(Error::DBError)
    }

    /// Returns an iterator over keys with the same prefix as the provided value.
    pub fn prefix_key_iterator<'a: 'b, 'b, K, V, P: AsRef<[u8]>>(
        &'a self,
        prefix: P,
    ) -> KeyIterator<'b, K>
    where
        K: ReadOnlyDBKey<V> + serde::de::DeserializeOwned,
    {
        let cf = K::cf(&self.cf);
        KeyIterator {
            inner: self.db.prefix_iterator_cf(cf, prefix),
            phantom: PhantomData,
        }
    }

    /// Returns an iterator over key-value pairs where the key has the same prefix
    /// with the provided value.
    pub fn prefix_iterator<'a: 'b, 'b, K, V, P: AsRef<[u8]>>(
        &'a self,
        prefix: P,
    ) -> KeyValueIterator<'b, K, V>
    where
        K: ReadOnlyDBKey<V> + serde::de::DeserializeOwned,
        V: serde::de::DeserializeOwned,
    {
        let cf = K::cf(&self.cf);
        KeyValueIterator {
            inner: self.db.prefix_iterator_cf(cf, prefix),
            phantom: PhantomData,
        }
    }
}

#[inline]
fn vec_push_merge(
    _: &[u8],
    existing_val: Option<&[u8]>,
    operands: &mut rocksdb::MergeOperands,
) -> Option<Vec<u8>> {
    let result = merge_push(existing_val, operands);
    Some(result)
}

pub struct KeyIterator<'a, K> {
    inner: rocksdb::DBIterator<'a>,
    phantom: PhantomData<K>,
}

impl<'a, K> Iterator for KeyIterator<'a, K>
where
    K: serde::de::DeserializeOwned,
{
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(k, _)| bincode::deserialize(k.as_ref()).unwrap())
    }
}

pub struct KeyValueIterator<'a, K, V> {
    inner: rocksdb::DBIterator<'a>,
    phantom: PhantomData<(K, V)>,
}

impl<'a, K, V> Iterator for KeyValueIterator<'a, K, V>
where
    K: serde::de::DeserializeOwned,
    V: serde::de::DeserializeOwned,
{
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, v)| {
            (
                bincode::deserialize(k.as_ref()).unwrap(),
                bincode::deserialize(v.as_ref()).unwrap(),
            )
        })
    }
}
