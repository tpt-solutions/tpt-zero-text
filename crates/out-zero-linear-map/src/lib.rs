#![no_std]
//! `out-zero-linear-map`: a small ordered map backed by a sorted sequence.
//!
//! Elements are kept sorted by key, so lookups use binary search (`O(log n)`)
//! and iteration yields keys in order. Insertion/removal are `O(n)` (elements
//! are shifted), which is ideal for small datasets where the linear cost is
//! dwarfed by the lack of heap overhead.
//!
//! - [`ArrayLinearMap<K, V, N>`] — fixed-capacity, `#![no_std]`, backed by an
//!   [`out_zero_arrayvec::ArrayVec`].
//! - [`VecLinearMap<K, V>`] (alloc feature) — unbounded, backed by `alloc::vec::Vec`.

use core::ops::Index;
use out_zero_arrayvec::ArrayVec;

/// A key/value pair stored in the map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entry<K, V> {
    /// The key.
    pub key: K,
    /// The value.
    pub value: V,
}

impl<K, V> Entry<K, V> {
    /// Construct an entry.
    pub const fn new(key: K, value: V) -> Self {
        Entry { key, value }
    }
}

/// Binary-search a sorted slice for `key`, returning the index or `None`.
fn search_sorted<K, V>(items: &[Entry<K, V>], key: &K) -> Option<usize>
where
    K: Ord,
{
    let mut lo = 0usize;
    let mut hi = items.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        match items[mid].key.cmp(key) {
            core::cmp::Ordering::Equal => return Some(mid),
            core::cmp::Ordering::Less => lo = mid + 1,
            core::cmp::Ordering::Greater => hi = mid,
        }
    }
    None
}

/// A fixed-capacity ordered map backed by a sorted `ArrayVec`.
pub struct ArrayLinearMap<K, V, const N: usize> {
    items: ArrayVec<Entry<K, V>, N>,
}

impl<K, V, const N: usize> ArrayLinearMap<K, V, N> {
    /// Create an empty map.
    pub const fn new() -> Self {
        ArrayLinearMap {
            items: ArrayVec::new(),
        }
    }

    /// Number of entries.
    #[inline]
    pub const fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the map is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Total capacity.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Get a reference to the value for `key`, if present.
    #[inline]
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        search_sorted(&self.items, key).map(|i| &self.items[i].value)
    }

    /// Get a mutable reference to the value for `key`, if present.
    #[inline]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: Ord,
    {
        search_sorted(&self.items, key).map(|i| &mut self.items[i].value)
    }

    /// Insert `key`/`value`. If the key exists, the value is replaced and the
    /// old value returned. Returns `Err(value)` (with the value handed back) if
    /// the map is full and the key is not already present.
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, V>
    where
        K: Ord,
    {
        if let Some(i) = search_sorted(&self.items, &key) {
            let old = core::mem::replace(&mut self.items[i].value, value);
            return Ok(Some(old));
        }
        let entry = Entry::new(key, value);
        // If the map is already full we cannot insert a new key; hand the value
        // back. `ArrayVec::insert` does the tail shift internally (sound).
        if self.items.is_full() {
            return Err(entry.value);
        }
        let idx = {
            let mut lo = 0usize;
            let mut hi = self.items.len();
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if self.items[mid].key < entry.key {
                    lo = mid + 1;
                } else {
                    hi = mid;
                }
            }
            lo
        };
        // `insert` returns Err only when full, which we ruled out above.
        let _ = self.items.insert(idx, entry);
        Ok(None)
    }

    /// Remove `key`, returning the removed value if present.
    #[inline]
    pub fn remove(&mut self, key: &K) -> Option<V>
    where
        K: Ord,
    {
        let i = search_sorted(&self.items, key)?;
        self.items.remove(i).map(|e| e.value)
    }

    /// Iterate over `(key, value)` pairs in key order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Entry<K, V>> {
        self.items.iter()
    }

    /// Iterate over keys in order.
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.items.iter().map(|e| &e.key)
    }

    /// Iterate over values in key order.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.items.iter().map(|e| &e.value)
    }
}

impl<K, V, const N: usize> Default for ArrayLinearMap<K, V, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, const N: usize> Index<&K> for ArrayLinearMap<K, V, N>
where
    K: Ord,
{
    type Output = V;
    #[inline]
    fn index(&self, key: &K) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

#[cfg(feature = "alloc")]
mod alloc_layer {
    extern crate alloc;
    use super::*;
    use alloc::vec::Vec;

    /// An unbounded ordered map backed by a sorted `Vec`.
    pub struct VecLinearMap<K, V> {
        items: Vec<Entry<K, V>>,
    }

    impl<K, V> VecLinearMap<K, V> {
        /// Create an empty map.
        pub fn new() -> Self {
            VecLinearMap { items: Vec::new() }
        }

        /// Number of entries.
        #[inline]
        pub fn len(&self) -> usize {
            self.items.len()
        }

        /// Whether the map is empty.
        #[inline]
        pub fn is_empty(&self) -> bool {
            self.items.is_empty()
        }

        /// Get a reference to the value for `key`, if present.
        #[inline]
        pub fn get(&self, key: &K) -> Option<&V>
        where
            K: Ord,
        {
            search_sorted(&self.items, key).map(|i| &self.items[i].value)
        }

        /// Get a mutable reference to the value for `key`, if present.
        #[inline]
        pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
        where
            K: Ord,
        {
            search_sorted(&self.items, key).map(|i| &mut self.items[i].value)
        }

        /// Insert `key`/`value`, replacing any existing value and returning it.
        #[inline]
        pub fn insert(&mut self, key: K, value: V) -> Option<V>
        where
            K: Ord,
        {
            if let Some(i) = search_sorted(&self.items, &key) {
                return Some(core::mem::replace(&mut self.items[i].value, value));
            }
            let mut lo = 0usize;
            let mut hi = self.items.len();
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if self.items[mid].key < key {
                    lo = mid + 1;
                } else {
                    hi = mid;
                }
            }
            self.items.insert(lo, Entry::new(key, value));
            None
        }

        /// Remove `key`, returning the removed value if present.
        #[inline]
        pub fn remove(&mut self, key: &K) -> Option<V>
        where
            K: Ord,
        {
            let i = search_sorted(&self.items, key)?;
            Some(self.items.remove(i).value)
        }

        /// Iterate over entries in key order.
        #[inline]
        pub fn iter(&self) -> impl Iterator<Item = &Entry<K, V>> {
            self.items.iter()
        }

        /// Iterate over keys in order.
        #[inline]
        pub fn keys(&self) -> impl Iterator<Item = &K> {
            self.items.iter().map(|e| &e.key)
        }

        /// Iterate over values in key order.
        #[inline]
        pub fn values(&self) -> impl Iterator<Item = &V> {
            self.items.iter().map(|e| &e.value)
        }
    }

    impl<K, V> Default for VecLinearMap<K, V> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<K, V> Index<&K> for VecLinearMap<K, V>
    where
        K: Ord,
    {
        type Output = V;
        #[inline]
        fn index(&self, key: &K) -> &V {
            self.get(key).expect("no entry found for key")
        }
    }
}

#[cfg(feature = "alloc")]
pub use alloc_layer::VecLinearMap;

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    #[test]
    fn insert_get_replace() {
        let mut m: ArrayLinearMap<i32, &str, 8> = ArrayLinearMap::new();
        assert!(m.insert(3, "c").is_ok());
        assert!(m.insert(1, "a").is_ok());
        assert!(m.insert(2, "b").is_ok());
        assert_eq!(m.get(&1), Some(&"a"));
        assert_eq!(m.get(&2), Some(&"b"));
        assert_eq!(m.get(&3), Some(&"c"));
        assert_eq!(m.get(&9), None);
        // keys are kept in order
        let mut keys = [0i32; 8];
        let mut n = 0;
        for k in m.keys() {
            keys[n] = *k;
            n += 1;
        }
        assert_eq!(&keys[..n], &[1, 2, 3]);
        // replace
        assert_eq!(m.insert(2, "B"), Ok(Some("b")));
        assert_eq!(m.get(&2), Some(&"B"));
    }

    #[test]
    fn remove_works() {
        let mut m: ArrayLinearMap<i32, i32, 8> = ArrayLinearMap::new();
        for i in 0..5 {
            m.insert(i, i * 10).ok();
        }
        assert_eq!(m.remove(&2), Some(20));
        assert_eq!(m.get(&2), None);
        assert_eq!(m.len(), 4);
    }

    #[test]
    fn overflow_errors() {
        let mut m: ArrayLinearMap<i32, i32, 2> = ArrayLinearMap::new();
        assert!(m.insert(1, 1).is_ok());
        assert!(m.insert(2, 2).is_ok());
        assert_eq!(m.insert(3, 3), Err(3));
    }

    #[test]
    fn index_trait() {
        let mut m: ArrayLinearMap<&str, i32, 4> = ArrayLinearMap::new();
        m.insert("x", 7).ok();
        assert_eq!(m[&"x"], 7);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn alloc_vec_map() {
        let mut m: VecLinearMap<i32, i32> = VecLinearMap::new();
        for i in 0..10 {
            assert_eq!(m.insert(i, i * 2), None);
        }
        assert_eq!(m.insert(5, 999), Some(10));
        assert_eq!(m.get(&5), Some(&999));
        assert_eq!(m.remove(&5), Some(999));
        assert_eq!(m.get(&5), None);
        let mut prev = -1;
        for k in m.keys() {
            assert!(*k > prev);
            prev = *k;
        }
    }
}

#[cfg(test)]
mod proptests {
    extern crate std;
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Sequence-of-ops oracle vs `BTreeMap`: every get after the same
        /// inserts/removes matches `std::collections::BTreeMap`.
        #[test]
        fn oracle_vs_btreemap(
            ops in proptest::collection::vec(
                (any::<u8>(), any::<u8>(), proptest::bool::ANY), 0..64
            )
        ) {
            use std::collections::BTreeMap;
            let mut map: ArrayLinearMap<u8, u8, 128> = ArrayLinearMap::new();
            let mut oracle: BTreeMap<u8, u8> = BTreeMap::new();
            for (k, v, rm) in ops {
                if rm {
                    prop_assert_eq!(map.remove(&k), oracle.remove(&k));
                } else {
                    // N=128 comfortably holds <=64 distinct keys, so insert
                    // never overflows and matches BTreeMap's `Option<V>` return.
                    let got = map.insert(k, v).unwrap_or_else(|_| unreachable!());
                    let want = oracle.insert(k, v);
                    prop_assert_eq!(got, want);
                }
            }
            for k in 0u8..=255 {
                prop_assert_eq!(map.get(&k), oracle.get(&k));
            }
            // Iteration order matches.
            let got: std::vec::Vec<u8> = map.keys().copied().collect();
            let want: std::vec::Vec<u8> = oracle.keys().copied().collect();
            prop_assert_eq!(got, want);
        }
    }
}
