use crate::Timestamp;
use std::collections::{BTreeMap, BTreeSet};

/// A map that drops its elements after an expiration time if they are
/// not accessed until then. This map always returns a clone of the value.
pub struct DropMap<K, V> {
    /// How many to-be-dropped elements are we allowed to hold until we
    /// trigger the GC.
    capacity: usize,
    to_drop_count: usize,
    data: BTreeMap<K, Entry<V>>,
    drop_queue: BTreeMap<Timestamp, SmallSet<K>>,
}

struct Entry<V> {
    value: V,
    expiration: Option<Timestamp>,
}

enum SmallSet<T> {
    Empty,
    Single(T),
    Double(T, T),
    Multi(BTreeSet<T>),
}

impl<T: Ord + Copy> SmallSet<T> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        match &self {
            SmallSet::Empty => true,
            _ => false,
        }
    }

    #[inline]
    pub fn insert(&mut self, item: T) {
        match self {
            SmallSet::Empty => {
                *self = SmallSet::Single(item);
            }
            SmallSet::Single(other) if other != &item => {
                *self = SmallSet::Double(*other, item);
            }
            SmallSet::Double(a, b) if a != &item && b != &item => {
                let mut set = BTreeSet::new();
                set.insert(*a);
                set.insert(*b);
                set.insert(item);
                *self = SmallSet::Multi(set);
            }
            SmallSet::Multi(set) => {
                set.insert(item);
            }
            _ => {}
        }
    }

    #[inline]
    pub fn remove(&mut self, item: &T) {
        match self {
            SmallSet::Single(other) if other == item => {
                *self = SmallSet::Empty;
            }
            SmallSet::Double(a, b) if a == item => {
                *self = SmallSet::Single(*b);
            }
            SmallSet::Double(a, b) if b == item => {
                *self = SmallSet::Single(*a);
            }
            SmallSet::Multi(set) => {
                set.remove(item);
                if set.len() == 2 {
                    let mut iter = set.iter();
                    let a = iter.next().unwrap();
                    let b = iter.next().unwrap();
                    *self = SmallSet::Double(*a, *b);
                }
            }
            _ => {}
        }
    }
}

impl<K: Ord + Copy, V: Clone> DropMap<K, V> {
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
        let data = self.data.entry(key).or_insert_with(|| Entry {
            value: f(),
            expiration: None,
        });

        // If the element has a expiration data (i.e it is supposed to be dropped)
        // cancel the drop and set the expiration data to None.
        if let Some(expiration) = data.expiration {
            cancel_drop(&mut self.drop_queue, &key, expiration);
            data.expiration = None;
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
                .or_insert_with(|| SmallSet::Empty)
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
fn cancel_drop<K: Ord + Copy>(
    map: &mut BTreeMap<Timestamp, SmallSet<K>>,
    key: &K,
    expiration: Timestamp,
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
