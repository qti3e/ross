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

    pub fn get<K: DbReadKey>(&self, key: K) -> Result<Option<K::Value>> {
        let cf = K::cf(&self.cf);
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
    pub fn push<K: DbWriteKey<Value = Vec<I>>, I: serde::Serialize>(
        &self,
        key: K,
        value: &I,
    ) -> Result<()> {
        let cf = K::cf(&self.cf);
        self.db
            .merge_cf(cf, serialize(key.key()), serialize(value))
            .map_err(Error::DBError)
    }

    /// Returns an iterator over keys with the same prefix as the provided value.
    /// One should prefer using `keys::Key::key_iterator(&db, prefix)` for simplicity.
    pub fn prefix_key_iterator<'a: 'b, 'b, K: DbKey, P: AsRef<[u8]>>(
        &'a self,
        prefix: P,
    ) -> KeyIterator<'b, K::Key> {
        let cf = K::cf(&self.cf);
        KeyIterator {
            inner: self.db.prefix_iterator_cf(cf, prefix),
            phantom: PhantomData,
        }
    }

    /// Returns an iterator over key-value pairs where the key has the same prefix
    /// with the provided value.
    /// One should prefer using `keys::Key::key_value_iterator(&db, prefix)` for simplicity.
    pub fn prefix_iterator<'a: 'b, 'b, K: DbReadKey, P: AsRef<[u8]>>(
        &'a self,
        prefix: P,
    ) -> KeyValueIterator<'b, K::Key, K::Value> {
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
