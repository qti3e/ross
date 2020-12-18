use super::bincode::{deserialize, serialize};
use super::iterator::*;
use super::keys::{self, DbKey, DbReadKey, DbWriteKey, CF};
use super::Batch;
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

    pub fn get<
        K: serde::Serialize + serde::de::DeserializeOwned,
        V: serde::Serialize + serde::de::DeserializeOwned,
        T,
    >(
        &self,
        key: T,
    ) -> Result<Option<V>>
    where
        T: DbReadKey<K, V>,
    {
        let cf = T::cf(&self.cf);
        let pinned = self
            .db
            .get_pinned_cf(cf, serialize(key.key()))
            .map_err(Error::DBError)?;
        let bytes = match pinned {
            Some(slice) => slice,
            None => return Ok(None),
        };
        let data = deserialize(bytes.as_ref());
        Ok(Some(data))
    }

    #[inline(always)]
    pub fn push<
        K: serde::Serialize + serde::de::DeserializeOwned,
        I: serde::Serialize + serde::de::DeserializeOwned,
        T,
    >(
        &self,
        key: T,
        value: &I,
    ) -> Result<()>
    where
        T: DbWriteKey<K, Vec<I>>,
    {
        let cf = T::cf(&self.cf);
        self.db
            .merge_cf(cf, serialize(key.key()), serialize(value))
            .map_err(Error::DBError)
    }

    /// Returns an iterator over keys with the same prefix as the provided value.
    /// One should prefer using `keys::Key::key_iterator(&db, prefix)` for simplicity.
    pub fn prefix_key_iterator<'a: 'b, 'b, K, T, P: AsRef<[u8]>>(
        &'a self,
        prefix: P,
    ) -> KeyIterator<'b, K>
    where
        K: serde::Serialize + serde::de::DeserializeOwned,
        T: DbKey<K>,
    {
        let cf = T::cf(&self.cf);
        KeyIterator {
            inner: self.db.prefix_iterator_cf(cf, prefix),
            phantom: PhantomData,
        }
    }

    /// Returns an iterator over key-value pairs where the key has the same prefix
    /// with the provided value.  
    /// One should prefer using `keys::Key::key_value_iterator(&db, prefix)` for simplicity.
    pub fn prefix_iterator<'a: 'b, 'b, K, V, T, P: AsRef<[u8]>>(
        &'a self,
        prefix: P,
    ) -> KeyValueIterator<'b, K, V>
    where
        K: serde::Serialize + serde::de::DeserializeOwned,
        V: serde::Serialize + serde::de::DeserializeOwned,
        T: DbReadKey<K, V>,
    {
        let cf = T::cf(&self.cf);
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
