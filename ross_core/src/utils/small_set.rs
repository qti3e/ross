use std::collections::HashSet;
use std::hash::Hash;

/// A set implementation that is optimized for when our set contains one or
/// two elements most of the times. It is used in DropMap.
pub enum SmallSet<T> {
    Empty,
    Single(T),
    Double(T, T),
    Multi(HashSet<T>),
}

impl<T: Copy + Hash + Eq> SmallSet<T> {
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
                let mut set = HashSet::with_capacity(4);
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

// TODO(qti3e) Test small-set.
