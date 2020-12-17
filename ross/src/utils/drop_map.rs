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
    pub(self) to_drop_count: usize,
    ttl: clock::Timestamp,
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
            ttl,
        }
    }

    /// Current number of elements in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }

    /// Return the element from the map with the given key or insert the one
    /// returned by the provided closure, the closure may fail in that case
    /// the error returned by the closure will be returned.
    #[inline]
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
    #[inline]
    pub fn drop(&mut self, key: K, now: clock::Timestamp) {
        if self.ttl == 0 {
            if let Some(entry) = self.data.remove(&key) {
                if let Some(e) = entry.expiration {
                    cancel_drop(&mut self.drop_queue, &key, e);
                    self.to_drop_count -= 1;
                }
            }
        } else {
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
        }
        if self.to_drop_count >= self.capacity {
            self.gc(now);
        }
    }

    /// Force run the garbage collector.
    #[inline]
    pub fn gc(&mut self, now: clock::Timestamp) {
        let to_delete = {
            let e = now + 1;
            let mut keep = self.drop_queue.split_off(&e);
            std::mem::swap(&mut self.drop_queue, &mut keep);
            keep
        };

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

#[cfg(test)]
mod test {
    use super::DropMap;
    use std::error::Error;
    use std::fmt;

    #[derive(Debug, PartialEq)]
    pub enum MapError {
        CustomError,
    }

    impl Error for MapError {}

    impl fmt::Display for MapError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    #[test]
    fn cache() {
        let mut count = 0;
        let mut getter = || -> Result<String, MapError> {
            count += 1;
            Ok(String::from("Hello"))
        };

        let mut map = DropMap::<i32, String>::new(3, 0);
        assert_eq!(
            map.get_or_maybe_insert_with(0, || getter()),
            Ok(&String::from("Hello"))
        );
        assert_eq!(
            map.get_or_maybe_insert_with(0, || getter()),
            Ok(&String::from("Hello"))
        );
        assert_eq!(count, 1);
    }

    #[test]
    fn error() {
        let getter = || -> Result<String, MapError> { Err(MapError::CustomError) };
        let mut map = DropMap::<i32, String>::new(3, 0);
        assert_eq!(
            map.get_or_maybe_insert_with(0, getter),
            Err(MapError::CustomError)
        );
    }

    #[test]
    fn gc() {
        let mut map = DropMap::<i32, i32>::new(2, 1);
        map.get_or_maybe_insert_with(0, || -> Result<i32, MapError> { Ok(1) })
            .unwrap();
        map.get_or_maybe_insert_with(1, || -> Result<i32, MapError> { Ok(10) })
            .unwrap();
        map.get_or_maybe_insert_with(2, || -> Result<i32, MapError> { Ok(10) })
            .unwrap();
        // Time = 1
        map.drop(0, 1);
        assert_eq!(map.len(), 3);
        // Time = 2
        map.drop(1, 2);
        assert_eq!(map.contains_key(&0), false);
        assert_eq!(map.contains_key(&1), true);
        assert_eq!(map.contains_key(&2), true);
        assert_eq!(map.len(), 2);
        assert_eq!(map.to_drop_count, 1);
        // Time = 3
        map.gc(3);
        assert_eq!(map.contains_key(&0), false);
        assert_eq!(map.contains_key(&1), false);
        assert_eq!(map.contains_key(&2), true);
        assert_eq!(map.len(), 1);
        assert_eq!(map.to_drop_count, 0);
    }
}
