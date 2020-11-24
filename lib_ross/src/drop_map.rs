use crate::Timestamp;
use std::collections::{BTreeMap, BTreeSet};

/// A map that drops it's elements after a expiration time if they are
/// not accessed until that time, this map always returns a clone of the
/// value.
pub struct DropMap<K, V> {
    /// How many to-be-dropped elements are we allowed to hold until we
    /// trigger the GC.
    capacity: usize,
    to_drop_count: usize,
    data: BTreeMap<K, Entry<V>>,
    drop_queue: BTreeMap<Timestamp, BTreeSet<K>>,
}

struct Entry<V> {
    value: V,
    expiration: Option<Timestamp>,
}

impl<K: Ord + Clone, V: Clone> DropMap<K, V> {
    pub fn new(capacity: usize) -> Self {
        DropMap {
            capacity,
            to_drop_count: 0,
            data: BTreeMap::new(),
            drop_queue: BTreeMap::new(),
        }
    }

    /// Return the element from the map with the given key or insert the one
    /// returned by the provided closure.
    pub fn get_or_insert_with<F: FnOnce() -> V>(&mut self, key: K, f: F) -> V {
        let data = self.data.entry(key.clone()).or_insert_with(|| Entry {
            value: f(),
            expiration: None,
        });

        // If the element has a expiration data (i.e it is supposed to be dropped)
        // cancel the drop and set the expiration data to None.
        if let Some(expiration) = data.expiration {
            data.expiration = None;
            cancel_drop(&mut self.drop_queue, &key, expiration);
            self.to_drop_count -= 1;
        }

        // This map always returns a clone of the data, it's supposed to work with
        // `RC`, `Arc`, `Box` and other smart pointers.
        data.value.clone()
    }

    /// Set an expiration date on the key, we will wait at least until the expiration
    /// time to actually drop the content.
    pub fn drop(&mut self, key: K, expiration: Timestamp) {
        if let Some(data) = self.data.get_mut(&key) {
            if let Some(e) = data.expiration {
                cancel_drop(&mut self.drop_queue, &key, e);
                self.to_drop_count -= 1;
            }
            data.expiration = Some(expiration);
            self.to_drop_count += 1;
            self.drop_queue
                .entry(expiration)
                .or_insert_with(|| BTreeSet::new())
                .insert(key);
        }

        if self.to_drop_count >= self.capacity {
            self.gc();
        }
    }

    /// Fore run the garbage collector.
    #[inline(always)]
    pub fn gc(&mut self) {
        let now = crate::now();
        let to_delete = self.drop_queue.split_off(&now);
        for (_, items) in to_delete {
            for key in items {
                self.to_drop_count -= 1;
                self.data.remove(&key);
            }
        }
    }
}

#[inline(always)]
fn cancel_drop<K: Ord>(map: &mut BTreeMap<Timestamp, BTreeSet<K>>, key: &K, expiration: Timestamp) {
    let mut remove = false;
    map.entry(expiration).and_modify(|e| {
        e.remove(key);
        remove = e.len() == 0;
    });
    if remove {
        map.remove(&expiration);
    }
}
