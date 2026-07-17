#![no_std]
//! `tpt-zero-arrayvec`: a `Vec`-like collection backed by a fixed-size array.
//!
//! [`ArrayVec<T, N>`] stores up to `N` elements inline. Pushing past capacity
//! is a no-panic error (`CapacityError`). `Drop` runs the inner elements' destructors
//! only for the initialized prefix, so it is sound under `#![no_std]`.

use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

/// Error returned when an `ArrayVec` is full.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CapacityError;

/// A `Vec`-like type backed by `[MaybeUninit<T>; N]`.
pub struct ArrayVec<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N> {
    /// Create an empty `ArrayVec`.
    pub const fn new() -> Self {
        // SAFETY: an array of `MaybeUninit` needs no initialization.
        ArrayVec {
            buf: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    /// Number of elements currently stored.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the vector is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether the vector has reached its capacity.
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    /// Total capacity.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Push `value`, returning a [`CapacityError`] if full.
    #[inline]
    pub fn push(&mut self, value: T) -> Result<(), CapacityError> {
        if self.len == N {
            return Err(CapacityError);
        }
        // SAFETY: `len` is < N, so this slot is uninitialized.
        self.buf[self.len].write(value);
        self.len += 1;
        Ok(())
    }

    /// Pop the last element, or `None` if empty.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: `len` now points at an initialized element.
        Some(unsafe { self.buf[self.len].assume_init_read() })
    }

    /// Insert `value` at `index`, shifting later elements right. Errors if full or
    /// `index > len`.
    #[inline]
    pub fn insert(&mut self, index: usize, value: T) -> Result<(), CapacityError> {
        if index > self.len {
            return Err(CapacityError);
        }
        if self.len == N {
            return Err(CapacityError);
        }
        let mut i = self.len;
        while i > index {
            // SAFETY: shifting initialized elements right; both slots valid.
            self.buf[i] = unsafe { MaybeUninit::new(self.buf[i - 1].assume_init_read()) };
            i -= 1;
        }
        self.buf[index].write(value);
        self.len += 1;
        Ok(())
    }

    /// Remove and return the element at `index`, shifting later elements left.
    /// Returns `None` if `index >= len`.
    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }
        // SAFETY: `index` is initialized.
        let value = unsafe { self.buf[index].assume_init_read() };
        let mut i = index;
        while i + 1 < self.len {
            self.buf[i] = unsafe { MaybeUninit::new(self.buf[i + 1].assume_init_read()) };
            i += 1;
        }
        self.len -= 1;
        Some(value)
    }

    /// Truncate to `new_len`, dropping trailing elements.
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        while self.len > new_len {
            self.len -= 1;
            // SAFETY: `len` points at an initialized element.
            unsafe { self.buf[self.len].assume_init_drop() };
        }
    }

    /// Remove all elements without deallocating capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Get a reference to the element at `index`.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            // SAFETY: `index < len` so the slot is initialized.
            Some(unsafe { self.buf[index].assume_init_ref() })
        } else {
            None
        }
    }

    /// Get a mutable reference to the element at `index`.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            // SAFETY: as above.
            Some(unsafe { self.buf[index].assume_init_mut() })
        } else {
            None
        }
    }

    /// Iterate over references.
    pub fn iter(&self) -> Iter<'_, T, N> {
        Iter {
            ptr: self.as_slice().as_ptr(),
            len: self.len,
            idx: 0,
            _marker: core::marker::PhantomData,
        }
    }

    /// Iterate over mutable references.
    pub fn iter_mut(&mut self) -> IterMut<'_, T, N> {
        let len = self.len;
        IterMut {
            ptr: self.as_slice_mut().as_mut_ptr(),
            len,
            idx: 0,
            _marker: core::marker::PhantomData,
        }
    }

    /// View the initialized contents as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: the first `len` slots are initialized.
        unsafe { core::slice::from_raw_parts(self.buf.as_ptr() as *const T, self.len) }
    }

    /// View the initialized contents as a mutable slice.
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        // SAFETY: as above.
        unsafe { core::slice::from_raw_parts_mut(self.buf.as_mut_ptr() as *mut T, self.len) }
    }
}

impl<T, const N: usize> Default for ArrayVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Deref for ArrayVec<T, N> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for ArrayVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_slice_mut()
    }
}

impl<T, const N: usize> AsRef<[T]> for ArrayVec<T, N> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Clone, const N: usize> Clone for ArrayVec<T, N> {
    fn clone(&self) -> Self {
        let mut out = ArrayVec::new();
        for item in self.iter() {
            // SAFETY: `item` is initialized and `out.len < N` because self.len <= N.
            out.buf[out.len].write(item.clone());
            out.len += 1;
        }
        out
    }
}

impl<T, const N: usize> Drop for ArrayVec<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T: PartialEq, const N: usize> PartialEq for ArrayVec<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq, const N: usize> Eq for ArrayVec<T, N> {}

impl<T: core::fmt::Debug, const N: usize> core::fmt::Debug for ArrayVec<T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_slice(), f)
    }
}

/// Immutable iterator over an [`ArrayVec`].
pub struct Iter<'a, T, const N: usize> {
    ptr: *const T,
    len: usize,
    idx: usize,
    _marker: core::marker::PhantomData<&'a ArrayVec<T, N>>,
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;
    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        if self.idx < self.len {
            // SAFETY: idx < len, so the slot is initialized and ptr is in bounds.
            let item = unsafe { &*self.ptr.add(self.idx) };
            self.idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

/// Mutable iterator over an [`ArrayVec`].
pub struct IterMut<'a, T, const N: usize> {
    ptr: *mut T,
    len: usize,
    idx: usize,
    _marker: core::marker::PhantomData<&'a mut ArrayVec<T, N>>,
}

impl<'a, T, const N: usize> Iterator for IterMut<'a, T, N> {
    type Item = &'a mut T;
    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        if self.idx < self.len {
            // SAFETY: idx < len, so the slot is initialized and ptr is in bounds.
            let item = unsafe { &mut *self.ptr.add(self.idx) };
            self.idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut v: ArrayVec<i32, 4> = ArrayVec::new();
        assert!(v.is_empty());
        assert!(v.push(1).is_ok());
        assert!(v.push(2).is_ok());
        assert_eq!(v.len(), 2);
        assert_eq!(v.pop(), Some(2));
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.pop(), None);
    }

    #[test]
    fn capacity_error() {
        let mut v: ArrayVec<i32, 2> = ArrayVec::new();
        assert!(v.push(1).is_ok());
        assert!(v.push(2).is_ok());
        assert_eq!(v.push(3), Err(CapacityError));
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn insert_remove() {
        let mut v: ArrayVec<i32, 4> = ArrayVec::new();
        v.push(1).unwrap();
        v.push(3).unwrap();
        v.insert(1, 2).unwrap();
        assert_eq!(v.as_slice(), &[1, 2, 3]);
        assert_eq!(v.remove(1), Some(2));
        assert_eq!(v.as_slice(), &[1, 3]);
        assert_eq!(v.remove(5), None);
    }

    #[test]
    fn truncate_clear() {
        let mut v: ArrayVec<i32, 4> = ArrayVec::new();
        for i in 0..4 {
            v.push(i).unwrap();
        }
        v.truncate(2);
        assert_eq!(v.as_slice(), &[0, 1]);
        v.clear();
        assert!(v.is_empty());
    }

    #[test]
    fn deref_and_iter() {
        let mut v: ArrayVec<i32, 3> = ArrayVec::new();
        v.push(10).unwrap();
        v.push(20).unwrap();
        let s: &[i32] = &v;
        assert_eq!(s, &[10, 20]);
        let sum: i32 = v.iter().sum();
        assert_eq!(sum, 30);
    }

    #[test]
    fn drop_runs_destructors() {
        use core::cell::Cell;
        struct DropCounter<'a>(&'a Cell<usize>);
        impl<'a> Drop for DropCounter<'a> {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let counter = Cell::new(0);
        {
            let mut v: ArrayVec<DropCounter<'_>, 3> = ArrayVec::new();
            v.push(DropCounter(&counter)).unwrap();
            v.push(DropCounter(&counter)).unwrap();
            v.pop(); // one dropped here
            // one remains, dropped on scope exit -> 2 total
        }
        assert_eq!(counter.get(), 2);
    }
}

#[cfg(test)]
mod proptests {
    extern crate alloc;
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn oracle_vs_vec(ops in proptest::collection::vec(
            (0..8usize, 0..1000i32), 0..200
        )) {
            let mut av: ArrayVec<i32, 16> = ArrayVec::new();
            let mut vv: alloc::vec::Vec<i32> = alloc::vec::Vec::new();
            for (idx, val) in ops {
                if vv.len() >= 16 {
                    continue;
                }
                let i = idx.min(vv.len());
                match av.insert(i, val) {
                    Ok(()) => vv.insert(i, val),
                    Err(_) => {
                        prop_assert_eq!(vv.len(), 16);
                    }
                }
            }
            prop_assert_eq!(av.as_slice(), &vv[..]);
        }
    }
}
