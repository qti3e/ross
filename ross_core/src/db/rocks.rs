use super::{Batch, DBKey, PartialDBKey};
use crate::utils::bincode_vec_push::merge_push;
use crate::{sync, Error, Result};

sync!(
    /// The read/write guard around the DB instance.
    DBSync(DB) {}
);

/// A type safe wrapper around rocksdb, it should not be considered thread-safe
/// and needs to be guarded by DBSync.
pub struct DB {
    db: rocksdb::DB,
}

impl DB {
    /// Open a new database instance.
    pub fn open(path: &str) -> Self {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.set_merge_operator("bincode-vec-append", vec_push_merge, None);
        let db = rocksdb::DB::open(&opts, path).unwrap();
        DB { db }
    }

    /// Perform the given transaction on the database.
    #[inline(always)]
    pub fn perform(&mut self, batch: Batch) -> Result<()> {
        self.db
            .write(batch.finalize())
            .map_err(|e| Error::DBError(e))
    }

    /// Return the data associated with the given key.
    pub fn get<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: DBKey<V>,
        V: serde::de::DeserializeOwned,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        let pinned = self.db.get_pinned(key).map_err(|e| Error::DBError(e))?;
        let bytes = match pinned {
            Some(slice) => slice,
            None => return Ok(None),
        };
        let data = bincode::deserialize(bytes.as_ref()).unwrap();
        Ok(Some(data))
    }

    /// Only deserialize the beginning of a value to match type V.
    /// For example if we are storing a `K -> (V, _)` we can just decode the `V`
    /// part and return it.
    pub fn get_partial<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: PartialDBKey<V>,
        V: serde::de::DeserializeOwned,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        let pinned = self.db.get_pinned(key).map_err(|e| Error::DBError(e))?;
        let bytes = match pinned {
            Some(slice) => slice,
            None => return Ok(None),
        };
        let data = bincode::deserialize(bytes.as_ref()).unwrap();
        Ok(Some(data))
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
