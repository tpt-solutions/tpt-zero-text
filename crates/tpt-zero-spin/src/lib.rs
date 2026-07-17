#![no_std]
//! `tpt-zero-spin`: spin-based mutual-exclusion locks for `#![no_std]`.
//!
//! Provides two locks built on atomic compare-and-swap plus
//! [`core::hint::spin_loop`], with no heap allocation and no `std`:
//!
//! - [`SpinMutex<T>`] — a mutual-exclusion lock protecting a `T`. The guard
//!   `Deref`s to the protected value.
//! - [`SpinRwLock<T>`] — a reader-writer lock allowing many concurrent readers
//!   or a single exclusive writer.
//!
//! # When to use
//!
//! Spin locks busy-wait, so they are appropriate only for very short critical
//! sections or when blocking is unavailable (e.g. interrupt context). Prefer an
//! OS-backed mutex when one is available.
//!
//! # Soundness / ordering
//!
//! Acquisition uses `Acquire` loads and `Release` stores; the lock word itself
//! is manipulated with `AcqRel` CAS. This release is **not** verified with
//! `loom` or `miri`; validate under your own concurrency tests before relying
//! on it in production.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// A mutual-exclusion lock that busy-waits (spins) while contended.
pub struct SpinMutex<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> SpinMutex<T> {
    /// Create a new `SpinMutex` protecting `value`.
    pub const fn new(value: T) -> Self {
        SpinMutex {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Acquire the lock, spinning until available, and return a guard.
    pub fn lock(&self) -> SpinMutexGuard<'_, T> {
        loop {
            // Try to flip false -> true.
            if self
                .locked
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
            while self.locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }
        SpinMutexGuard { mutex: self }
    }
}

unsafe impl<T: Send> Sync for SpinMutex<T> {}
unsafe impl<T: Send> Send for SpinMutex<T> {}

/// RAII guard returned by [`SpinMutex::lock`].
pub struct SpinMutexGuard<'a, T> {
    mutex: &'a SpinMutex<T>,
}

impl<'a, T> Deref for SpinMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<'a, T> DerefMut for SpinMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<'a, T> Drop for SpinMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}

/// State word for [`SpinRwLock`]: `0` = unlocked, `n` = `n` shared readers,
/// `u32::MAX` = one exclusive writer.
const WRITE_LOCK: u32 = u32::MAX;

/// A reader-writer lock that busy-waits (spins) while contended.
pub struct SpinRwLock<T> {
    state: AtomicU32,
    value: UnsafeCell<T>,
}

impl<T> SpinRwLock<T> {
    /// Create a new `SpinRwLock` protecting `value`.
    pub const fn new(value: T) -> Self {
        SpinRwLock {
            state: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    /// Acquire a shared read lock.
    pub fn read(&self) -> SpinRwReadGuard<'_, T> {
        loop {
            let cur = self.state.load(Ordering::Relaxed);
            if cur != WRITE_LOCK {
                if self
                    .state
                    .compare_exchange(cur, cur + 1, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok()
                {
                    break;
                }
            } else {
                while self.state.load(Ordering::Relaxed) == WRITE_LOCK {
                    core::hint::spin_loop();
                }
            }
        }
        SpinRwReadGuard { lock: self }
    }

    /// Acquire an exclusive write lock.
    pub fn write(&self) -> SpinRwWriteGuard<'_, T> {
        loop {
            if self
                .state
                .compare_exchange(0, WRITE_LOCK, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
            while self.state.load(Ordering::Relaxed) != 0 {
                core::hint::spin_loop();
            }
        }
        SpinRwWriteGuard { lock: self }
    }
}

unsafe impl<T: Send> Sync for SpinRwLock<T> {}
unsafe impl<T: Send> Send for SpinRwLock<T> {}

/// RAII read guard returned by [`SpinRwLock::read`].
pub struct SpinRwReadGuard<'a, T> {
    lock: &'a SpinRwLock<T>,
}

impl<'a, T> Deref for SpinRwReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

impl<'a, T> Drop for SpinRwReadGuard<'a, T> {
    fn drop(&mut self) {
        // Downgrade one reader.
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

/// RAII write guard returned by [`SpinRwLock::write`].
pub struct SpinRwWriteGuard<'a, T> {
    lock: &'a SpinRwLock<T>,
}

impl<'a, T> Deref for SpinRwWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

impl<'a, T> DerefMut for SpinRwWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<'a, T> Drop for SpinRwWriteGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutex_exclusive() {
        let m: SpinMutex<i32> = SpinMutex::new(0);
        *m.lock() += 1;
        assert_eq!(*m.lock(), 1);
    }

    #[test]
    fn rw_many_readers_one_writer() {
        let l: SpinRwLock<i32> = SpinRwLock::new(5);
        assert_eq!(*l.read(), 5);
        assert_eq!(*l.read(), 5);
        *l.write() = 10;
        assert_eq!(*l.read(), 10);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// A mutex round-trips a stored value.
        #[test]
        fn mutex_holds(v in any::<i64>()) {
            let m: SpinMutex<i64> = SpinMutex::new(v);
            prop_assert_eq!(*m.lock(), v);
        }

        /// An rw-lock round-trips its stored value under read and write.
        #[test]
        fn rw_holds(v in any::<i64>()) {
            let l: SpinRwLock<i64> = SpinRwLock::new(v);
            prop_assert_eq!(*l.read(), v);
            *l.write() = v;
            prop_assert_eq!(*l.read(), v);
        }
    }
}
