use super::clock;
use super::small_set::SmallSet;
use std::collections::{hash_map, BTreeMap, HashMap};
use std::fmt;
use std::hash::Hash;

/// A map that drops its elements after an expiration time, if they are
/// accessed until then the expiration time gets invalidated.
pub struct DropMap<K, V> {
    data: HashMap<K, Entry<V>>,
    /// How many to-be-dropped elements are we allowed to hold until we
    /// trigger the GC.
    capacity: usize,
    drop_queue: BTreeMap<clock::Timestamp, SmallSet<K>>,
    to_drop_count: usize,
    ttl: clock::Timestamp
}

struct Entry<V> {
    value: V,
    expiration: Option<clock::Timestamp>,
}

impl<K: Copy + Hash + Eq, V: Clone> DropMap<K, V> {
    pub fn new(capacity: usize, ttl: clock::Timestamp) -> Self {
        DropMap {
            data: HashMap::with_capacity(capacity + 1),
            capacity,
            drop_queue: BTreeMap::new(),
            to_drop_count: 0,
            ttl
        }
    }

    /// Current number of elements in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Return the element from the map with the given key or insert the one
    /// returned by the provided closure, the closure may fail in that case
    /// the error returned by the closure will be returned.
    pub fn get_or_maybe_insert_with<F: FnOnce() -> Result<V, E>, E: fmt::Debug>(
        &mut self,
        key: K,
        f: F,
    ) -> Result<&V, E> {
        let data = match self.data.entry(key) {
            hash_map::Entry::Occupied(o) => o.into_mut(),
            hash_map::Entry::Vacant(v) => {
                let value = f()?;
                v.insert(Entry {
                    value,
                    expiration: None,
                })
            }
        };

        // If the element has a expiration date (i.e it is supposed to be dropped)
        // cancel the drop and set the expiration data to None.
        if let Some(expiration) = data.expiration {
            cancel_drop(&mut self.drop_queue, &key, expiration);
            data.expiration = None;
            self.to_drop_count -= 1;
        }

        // This map always returns a clone of the data, it's supposed to work with
        // `RC`, `Arc`, `Box` and other smart pointers.
        Ok(&data.value)
    }

    /// Set an expiration date on the key, we will wait at least until the expiration
    /// time to actually drop the content.
    pub fn drop(&mut self, key: K, now: clock::Timestamp) {
        let expiration = now + self.ttl;
        if let Some(data) = self.data.get_mut(&key) {
            if let Some(e) = data.expiration {
                cancel_drop(&mut self.drop_queue, &key, e);
                self.to_drop_count -= 1;
            }
            data.expiration = Some(expiration);
            self.to_drop_count += 1;
            self.drop_queue
                .entry(expiration)
                .or_insert_with(|| SmallSet::Empty)
                .insert(key);
        }

        if self.to_drop_count >= self.capacity {
            self.gc(now);
        }
    }

    /// Force run the garbage collector.
    #[inline]
    pub fn gc(&mut self, now: clock::Timestamp) {
        let to_delete = self.drop_queue.split_off(&now);
        for (_, items) in to_delete {
            match items {
                SmallSet::Empty => {}
                SmallSet::Single(key) => {
                    self.to_drop_count -= 1;
                    self.data.remove(&key);
                }
                SmallSet::Double(k1, k2) => {
                    self.to_drop_count -= 2;
                    self.data.remove(&k1);
                    self.data.remove(&k2);
                }
                SmallSet::Multi(set) => {
                    self.to_drop_count -= set.len();
                    for key in set {
                        self.data.remove(&key);
                    }
                }
            }
        }
    }
}

#[inline(always)]
fn cancel_drop<K: Eq + Hash + Copy>(
    map: &mut BTreeMap<clock::Timestamp, SmallSet<K>>,
    key: &K,
    expiration: clock::Timestamp,
) {
    let mut remove = false;
    map.entry(expiration).and_modify(|e| {
        e.remove(key);
        remove = e.is_empty();
    });
    if remove {
        map.remove(&expiration);
    }
}
