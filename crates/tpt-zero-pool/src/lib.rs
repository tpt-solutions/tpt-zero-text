#![no_std]
//! `tpt-zero-pool`: a fixed-capacity object pool with RAII guards.
//!
//! A pool pre-allocates `N` slots and hands them out via [`ArrayPool::acquire`],
//! returning a [`PoolGuard`] that **owns** a slot for its lifetime and returns
//! it to the free list (running the value's destructor) on drop. This avoids
//! repeated heap allocation and fragmentation in `no_std` environments.
//!
//! The pool is **`!Sync` by design** (it manages internal mutable slot state
//! without locking). To share a pool across threads, wrap it in a
//! `tpt-zero-spin` `SpinMutex`. See the crate README for guidance.
//!
//! - [`ArrayPool<T, N>`] — fixed-capacity, `#![no_std]`.
//! - [`Pool<T>`] (alloc feature) — unbounded `Vec`-backed pool.

use core::cell::{Cell, UnsafeCell};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

/// A fixed-capacity object pool.
///
/// Not `Sync`: internal slot bookkeeping is unsynchronized (it uses interior
/// mutability via `Cell`/`UnsafeCell`). Compose with a `tpt-zero-spin`
/// `SpinMutex` to share across threads.
pub struct ArrayPool<T, const N: usize> {
    values: [UnsafeCell<MaybeUninit<T>>; N],
    occupied: [Cell<bool>; N],
    // `free_stack[0..free_top]` hold `Some(slot_index)` for free slots.
    free_stack: [Cell<Option<usize>>; N],
    free_top: Cell<usize>,
    _not_sync: PhantomData<Cell<()>>,
}

impl<T, const N: usize> ArrayPool<T, N> {
    /// Create a pool with `N` empty slots.
    pub fn new() -> Self {
        // SAFETY: `UnsafeCell<MaybeUninit<T>>`, `Cell<bool>` and
        // `Cell<Option<usize>>` are valid in any bit pattern; we then explicitly
        // initialize the cells below.
        let pool: ArrayPool<T, N> = unsafe { MaybeUninit::zeroed().assume_init() };
        for i in 0..N {
            pool.occupied[i].set(false);
            // Push indices so acquire hands out slot 0 first.
            pool.free_stack[i].set(Some(N - 1 - i));
        }
        pool.free_top.set(N);
        pool
    }

    /// Number of currently-acquired (occupied) slots.
    #[inline]
    pub fn len(&self) -> usize {
        N - self.free_top.get()
    }

    /// Whether all slots are free.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.free_top.get() == N
    }

    /// Total capacity.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Acquire a slot, initializing it with `value`.
    ///
    /// Returns `None` if the pool is full. The returned [`PoolGuard`] returns
    /// the slot (and drops `value`) when it goes out of scope.
    pub fn acquire(&self, value: T) -> Option<PoolGuard<'_, T, N>> {
        let top = self.free_top.get();
        if top == 0 {
            return None;
        }
        let idx = self.free_stack[top - 1].get().expect("free slot missing");
        self.free_top.set(top - 1);
        // SAFETY: `idx` is a unique free slot; write the value and mark occupied.
        unsafe {
            (*self.values[idx].get()).write(value);
        }
        self.occupied[idx].set(true);
        Some(PoolGuard {
            pool: self,
            index: idx,
        })
    }
}

impl<T, const N: usize> Default for ArrayPool<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for an acquired pool slot. Derefs to `T`; on drop the value is
/// destroyed and the slot returned to the free list.
pub struct PoolGuard<'a, T, const N: usize> {
    pool: &'a ArrayPool<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> PoolGuard<'a, T, N> {
    /// The slot index this guard occupies.
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Give the slot (and its value) back to the pool early. The guard becomes
    /// inert and must not be used afterwards.
    pub fn release(mut self) {
        self.free_slot();
        // Prevent `Drop` from running the cleanup a second time.
        core::mem::forget(self);
    }

    /// Drop the slot's value and return the index to the free list.
    ///
    fn free_slot(&mut self) {
        unsafe {
            (*self.pool.values[self.index].get()).assume_init_drop();
            self.pool.occupied[self.index].set(false);
            let top = self.pool.free_top.get();
            self.pool.free_stack[top].set(Some(self.index));
            self.pool.free_top.set(top + 1);
        }
    }
}

impl<'a, T, const N: usize> Deref for PoolGuard<'a, T, N> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: the slot is occupied for the guard's lifetime.
        unsafe { (*self.pool.values[self.index].get()).assume_init_ref() }
    }
}

impl<'a, T, const N: usize> DerefMut for PoolGuard<'a, T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the slot is occupied for the guard's lifetime.
        unsafe { &mut *(*self.pool.values[self.index].get()).assume_init_mut() }
    }
}

impl<'a, T, const N: usize> Drop for PoolGuard<'a, T, N> {
    fn drop(&mut self) {
        self.free_slot();
    }
}

#[cfg(feature = "alloc")]
mod alloc_layer {
    extern crate alloc;
    use super::*;
    use alloc::vec::Vec;
    use core::cell::RefCell;

    struct Inner<T> {
        values: Vec<UnsafeCell<MaybeUninit<T>>>,
        occupied: Vec<bool>,
        free: Vec<usize>,
    }

    /// An unbounded object pool backed by a `Vec`.
    pub struct Pool<T> {
        inner: RefCell<Inner<T>>,
        _not_sync: PhantomData<Cell<()>>,
    }

    impl<T> Pool<T> {
        /// Create an empty pool.
        pub fn new() -> Self {
            Pool {
                inner: RefCell::new(Inner {
                    values: Vec::new(),
                    occupied: Vec::new(),
                    free: Vec::new(),
                }),
                _not_sync: PhantomData,
            }
        }

        /// Number of currently-acquired (occupied) slots.
        #[inline]
        pub fn len(&self) -> usize {
            let inner = self.inner.borrow();
            inner.values.len() - inner.free.len()
        }

        /// Whether all slots are free.
        #[inline]
        pub fn is_empty(&self) -> bool {
            let inner = self.inner.borrow();
            inner.free.len() == inner.values.len()
        }

        /// Acquire a slot, initializing it with `value`. Grows the backing
        /// `Vec` if no free slot is available.
        pub fn acquire(&self, value: T) -> PoolGuardAlloc<'_, T> {
            let mut inner = self.inner.borrow_mut();
            let index = match inner.free.pop() {
                Some(i) => i,
                None => {
                    let i = inner.values.len();
                    inner.values.push(UnsafeCell::new(MaybeUninit::uninit()));
                    inner.occupied.push(true);
                    i
                }
            };
            unsafe {
                (*inner.values[index].get()).write(value);
            }
            inner.occupied[index] = true;
            drop(inner);
            PoolGuardAlloc { pool: self, index }
        }
    }

    impl<T> Default for Pool<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    /// RAII guard for an acquired `Pool` slot.
    pub struct PoolGuardAlloc<'a, T> {
        pool: &'a Pool<T>,
        index: usize,
    }

    impl<'a, T> PoolGuardAlloc<'a, T> {
        /// The slot index this guard occupies.
        #[inline]
        pub fn index(&self) -> usize {
            self.index
        }
    }

    impl<'a, T> Deref for PoolGuardAlloc<'a, T> {
        type Target = T;
        #[inline]
        fn deref(&self) -> &T {
            let inner = self.pool.inner.borrow();
            unsafe { (*inner.values[self.index].get()).assume_init_ref() }
        }
    }

    impl<'a, T> DerefMut for PoolGuardAlloc<'a, T> {
        #[inline]
        fn deref_mut(&mut self) -> &mut T {
            let inner = self.pool.inner.borrow_mut();
            unsafe { &mut *(*inner.values[self.index].get()).assume_init_mut() }
        }
    }

    impl<'a, T> Drop for PoolGuardAlloc<'a, T> {
        fn drop(&mut self) {
            let mut inner = self.pool.inner.borrow_mut();
            unsafe {
                (*inner.values[self.index].get()).assume_init_drop();
            }
            inner.occupied[self.index] = false;
            inner.free.push(self.index);
        }
    }
}

#[cfg(feature = "alloc")]
pub use alloc_layer::{Pool, PoolGuardAlloc};

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::cell::Cell;

    #[test]
    fn acquire_release_drops() {
        struct Track<'a>(&'a Cell<usize>);
        impl<'a> Drop for Track<'a> {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let drops = Cell::new(0);
        let pool: ArrayPool<Track<'_>, 3> = ArrayPool::new();
        assert!(pool.is_empty());
        let g1 = pool.acquire(Track(&drops)).unwrap();
        assert_eq!(g1.index(), 0);
        drop(g1);
        assert_eq!(drops.get(), 1);
        assert!(pool.is_empty());
    }

    #[test]
    fn full_pool_returns_none() {
        let pool: ArrayPool<i32, 2> = ArrayPool::new();
        let _a = pool.acquire(1).unwrap();
        let _b = pool.acquire(2).unwrap();
        assert!(pool.acquire(3).is_none());
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn reuse_slot_after_release() {
        let pool: ArrayPool<i32, 2> = ArrayPool::new();
        {
            let g = pool.acquire(10).unwrap();
            assert_eq!(*g, 10);
        }
        let g2 = pool.acquire(20).unwrap();
        assert_eq!(*g2, 20);
        assert_eq!(g2.index(), 0);
    }

    #[test]
    fn explicit_release() {
        let pool: ArrayPool<i32, 2> = ArrayPool::new();
        let g = pool.acquire(5).unwrap();
        g.release();
        let g2 = pool.acquire(7).unwrap();
        assert_eq!(*g2, 7);
    }

    #[test]
    fn mutate_through_guard() {
        let pool: ArrayPool<i32, 2> = ArrayPool::new();
        let mut g = pool.acquire(1).unwrap();
        *g += 41;
        assert_eq!(*g, 42);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn alloc_pool_grows() {
        let pool: Pool<i32> = Pool::new();
        let mut guards = std::vec::Vec::new();
        for i in 0..5 {
            guards.push(pool.acquire(i));
        }
        assert_eq!(pool.len(), 5);
        guards.clear();
        assert!(pool.is_empty());
        let g = pool.acquire(99);
        assert_eq!(*g, 99);
    }
}

#[cfg(test)]
mod proptests {
    extern crate std;
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Acquiring up to `N` values never panics; releasing all frees the pool.
        #[test]
        fn acquire_release_bookkeeping(ops in proptest::collection::vec(any::<u8>(), 0..200)) {
            let pool: ArrayPool<u8, 16> = ArrayPool::new();
            let mut live: std::vec::Vec<PoolGuard<u8, 16>> = std::vec::Vec::new();
            for &v in &ops {
                if live.len() < 16 {
                    live.push(pool.acquire(v).unwrap());
                } else {
                    live.pop();
                }
                prop_assert!(live.len() <= 16);
            }
            live.clear();
            prop_assert!(pool.is_empty());
        }
    }
}
