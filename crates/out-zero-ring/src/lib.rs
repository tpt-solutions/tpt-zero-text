#![no_std]
//! `out-zero-ring`: a fixed-capacity circular buffer (`Ring`) for streaming data.
//!
//! - [`Ring<T, N>`] holds up to `N` elements inline.
//! - `push_back` appends; when full it can either error or, in `overwrite` mode,
//!   drop the oldest element.
//! - `pop_front` removes the oldest element.
//! - `iter` traverses oldest-to-newest without moving data.

use core::mem::MaybeUninit;

/// Error returned when the ring is full and not in overwrite mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RingFull;

/// A fixed-capacity circular buffer.
pub struct Ring<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    /// Index of the oldest element (or equal to `tail` when empty).
    head: usize,
    /// One past the newest element (mod N).
    tail: usize,
    /// Number of initialized elements.
    len: usize,
    /// When `true`, pushing onto a full ring overwrites the oldest element.
    overwrite: bool,
}

impl<T, const N: usize> Ring<T, N> {
    /// Create an empty ring in error-on-full mode.
    pub const fn new() -> Self {
        Ring {
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            head: 0,
            tail: 0,
            len: 0,
            overwrite: false,
        }
    }

    /// Create an empty ring in overwrite (drop-oldest) mode.
    pub const fn new_overwrite() -> Self {
        let mut r = Self::new();
        r.overwrite = true;
        r
    }

    /// Number of elements currently stored.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the ring is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether the ring is full.
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    /// Capacity.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Append `value` to the back. In error mode, returns `RingFull` when full;
    /// in overwrite mode, drops the oldest element and stores `value`.
    #[inline]
    pub fn push_back(&mut self, value: T) -> Result<(), RingFull> {
        if self.len == N {
            if !self.overwrite {
                return Err(RingFull);
            }
            // Drop the oldest element to free a slot.
            // SAFETY: head points at an initialized element.
            unsafe { self.buf[self.head].assume_init_drop() };
            self.head = (self.head + 1) % N;
            self.len -= 1;
        }
        // SAFETY: tail points at an uninitialized slot (len < N).
        self.buf[self.tail].write(value);
        self.tail = (self.tail + 1) % N;
        self.len += 1;
        Ok(())
    }

    /// Remove and return the oldest element, or `None` if empty.
    #[inline]
    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        // SAFETY: head points at an initialized element.
        let value = unsafe { self.buf[self.head].assume_init_read() };
        self.head = (self.head + 1) % N;
        self.len -= 1;
        if self.len == 0 {
            self.head = 0;
            self.tail = 0;
        }
        Some(value)
    }

    /// Reference to the oldest element, or `None`.
    #[inline]
    pub fn front(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        // SAFETY: head points at an initialized element.
        Some(unsafe { self.buf[self.head].assume_init_ref() })
    }

    /// Mutable reference to the oldest element, or `None`.
    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            return None;
        }
        Some(unsafe { self.buf[self.head].assume_init_mut() })
    }

    /// Reference to the newest element, or `None`.
    #[inline]
    pub fn back(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        let idx = (self.tail + N - 1) % N;
        Some(unsafe { self.buf[idx].assume_init_ref() })
    }

    /// Remove all elements, dropping them.
    #[inline]
    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    /// Iterate over elements oldest-to-newest.
    pub fn iter(&self) -> Iter<'_, T, N> {
        Iter { ring: self, idx: 0 }
    }
}

impl<T, const N: usize> Default for Ring<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Drop for Ring<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T: Clone, const N: usize> Clone for Ring<T, N> {
    fn clone(&self) -> Self {
        let mut out = if self.overwrite {
            Ring::new_overwrite()
        } else {
            Ring::new()
        };
        for item in self.iter() {
            // SAFETY: len <= N so push_back cannot fail.
            out.push_back(item.clone()).ok();
        }
        out
    }
}

impl<T: PartialEq, const N: usize> PartialEq for Ring<T, N> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        self.iter().eq(other.iter())
    }
}

impl<T: Eq, const N: usize> Eq for Ring<T, N> {}

impl<T: core::fmt::Debug, const N: usize> core::fmt::Debug for Ring<T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// Immutable iterator over a [`Ring`].
pub struct Iter<'a, T, const N: usize> {
    ring: &'a Ring<T, N>,
    idx: usize,
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;
    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        if self.idx < self.ring.len {
            let slot = (self.ring.head + self.idx) % N;
            // SAFETY: idx < len, so the slot is initialized.
            let item = unsafe { self.ring.buf[slot].assume_init_ref() };
            self.idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;

    #[test]
    fn basic_push_pop() {
        let mut r: Ring<i32, 3> = Ring::new();
        assert!(r.is_empty());
        r.push_back(1).unwrap();
        r.push_back(2).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r.front(), Some(&1));
        assert_eq!(r.back(), Some(&2));
        assert_eq!(r.pop_front(), Some(1));
        assert_eq!(r.pop_front(), Some(2));
        assert_eq!(r.pop_front(), None);
    }

    #[test]
    fn full_errors() {
        let mut r: Ring<i32, 2> = Ring::new();
        r.push_back(1).unwrap();
        r.push_back(2).unwrap();
        assert_eq!(r.push_back(3), Err(RingFull));
    }

    #[test]
    fn overwrite_mode() {
        let mut r: Ring<i32, 2> = Ring::new_overwrite();
        r.push_back(1).unwrap();
        r.push_back(2).unwrap();
        r.push_back(3).unwrap(); // drops 1
        assert_eq!(r.pop_front(), Some(2));
        assert_eq!(r.pop_front(), Some(3));
        assert_eq!(r.pop_front(), None);
    }

    #[test]
    fn clear_drops() {
        use core::cell::Cell;
        struct C<'a>(&'a Cell<usize>);
        impl<'a> Drop for C<'a> {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let c = Cell::new(0);
        {
            let mut r: Ring<C<'_>, 3> = Ring::new();
            r.push_back(C(&c)).unwrap();
            r.push_back(C(&c)).unwrap();
            r.clear(); // 2 dropped
            r.push_back(C(&c)).unwrap();
            // 1 dropped on scope exit
        }
        assert_eq!(c.get(), 3);
    }

    #[test]
    fn iter_order() {
        let mut r: Ring<i32, 4> = Ring::new();
        r.push_back(10).unwrap();
        r.push_back(20).unwrap();
        r.pop_front();
        r.push_back(30).unwrap();
        let collected: alloc::vec::Vec<i32> = r.iter().copied().collect();
        assert_eq!(collected, alloc::vec![20, 30]);
    }
}

#[cfg(test)]
mod proptests {
    extern crate alloc;
    use super::*;
    use proptest::prelude::*;

    #[derive(Debug)]
    enum Op {
        Push(i32),
        Pop,
    }

    proptest! {
        #[test]
        fn oracle_vs_vecdeque(ops in proptest::collection::vec(
            prop_oneof![
                (0..1000i32).prop_map(Op::Push),
                proptest::bool::ANY.prop_map(|_| Op::Pop),
            ],
            0..300
        )) {
            let mut ring: Ring<i32, 16> = Ring::new();
            let mut dq: alloc::collections::VecDeque<i32> = alloc::collections::VecDeque::new();
            for op in ops {
                match op {
                    Op::Push(v) => {
                        if dq.len() >= 16 {
                            continue;
                        }
                        match ring.push_back(v) {
                            Ok(()) => dq.push_back(v),
                            Err(_) => prop_assert_eq!(dq.len(), 16),
                        }
                    }
                    Op::Pop => {
                        prop_assert_eq!(ring.pop_front(), dq.pop_front());
                    }
                }
            }
            let ring_items: alloc::vec::Vec<i32> = ring.iter().copied().collect();
            let dq_items: alloc::vec::Vec<i32> = dq.iter().copied().collect();
            prop_assert_eq!(ring_items, dq_items);
        }
    }
}
