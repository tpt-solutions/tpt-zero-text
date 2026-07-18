#![no_std]
//! `out-zero-once`: one-time initialization primitives for `#![no_std]`.
//!
//! Provides three building blocks, all built on a small lock-free state
//! machine with no heap allocation and no `std`:
//!
//! - [`Once`] — a flag that runs an initialization closure at most once.
//! - [`OnceCell<T>`] — a cell holding a `T` that is initialized exactly once.
//! - [`Lazy<T, F>`] — a value computed on first access (the closure is stored
//!   inline, so no `alloc` is required).
//!
//! # Soundness / ordering
//!
//! The state machine uses [`core::sync::atomic`] with `Acquire`/`Release`
//! ordering so that the initialization write is visible to every caller that
//! observes the "done" state. It is **not** verified with `loom` or `miri` in
//! this release; treat it as best-effort and validate under your own
//! concurrency tests before relying on it in production.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};

/// State values for the [`Once`] state machine.
const INCOMPLETE: u8 = 0;
const RUNNING: u8 = 1;
const COMPLETE: u8 = 2;

/// A synchronization primitive for running one-time initialization.
///
/// `call_once` runs `f` exactly once across all threads; concurrent callers
/// block (via a spin loop) until the first completes.
pub struct Once {
    state: AtomicU8,
}

impl Once {
    /// Create a new, uninitialized `Once`.
    pub const fn new() -> Self {
        Once {
            state: AtomicU8::new(INCOMPLETE),
        }
    }

    /// Run `f` if this `Once` has not yet been initialized.
    ///
    /// If another thread is currently initializing, this call spins until the
    /// initialization completes, then returns without running `f` again.
    pub fn call_once<F: FnOnce()>(&self, f: F) {
        // Fast path: already complete.
        if self.state.load(Ordering::Acquire) == COMPLETE {
            return;
        }
        self.slow_call_once(f);
    }

    fn slow_call_once<F: FnOnce()>(&self, f: F) {
        // Try to claim the RUNNING slot.
        match self
            .state
            .compare_exchange(INCOMPLETE, RUNNING, Ordering::Acquire, Ordering::Acquire)
        {
            Ok(_) => {
                f();
                self.state.store(COMPLETE, Ordering::Release);
            }
            Err(COMPLETE) => {}
            Err(RUNNING) => {
                // Another thread is initializing; wait for COMPLETE.
                while self.state.load(Ordering::Acquire) == RUNNING {
                    core::hint::spin_loop();
                }
            }
            Err(_) => unreachable!("state can only be INCOMPLETE/RUNNING/COMPLETE"),
        }
    }

    /// Whether initialization has completed.
    pub fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == COMPLETE
    }
}

impl Default for Once {
    fn default() -> Self {
        Self::new()
    }
}

/// A cell that can be written to exactly once.
///
/// `get_or_init` initializes the cell on first call and returns a reference to
/// the stored value. The value lives for the lifetime of the cell.
pub struct OnceCell<T> {
    once: Once,
    value: UnsafeCell<Option<T>>,
}

impl<T> OnceCell<T> {
    /// Create an empty cell.
    pub const fn new() -> Self {
        OnceCell {
            once: Once::new(),
            value: UnsafeCell::new(None),
        }
    }

    /// Get the value, initializing it with `f` if necessary.
    ///
    /// # Panics
    ///
    /// If `f` panics, the cell remains uninitialized and a later call will
    /// retry initialization (unlike `std::sync::Once`, which would poison).
    pub fn get_or_init<F: FnOnce() -> T>(&self, f: F) -> &T {
        self.once.call_once(|| unsafe {
            *self.value.get() = Some(f());
        });
        // Safety: after `call_once`, the `Some` variant is stable for `'a`.
        unsafe { (*self.value.get()).as_ref().unwrap_unchecked() }
    }

    /// Get the value if already initialized, without initializing.
    pub fn get(&self) -> Option<&T> {
        if self.once.is_completed() {
            // Safety: completed means the `Some` variant is stable.
            unsafe { (*self.value.get()).as_ref() }
        } else {
            None
        }
    }
}

impl<T> Default for OnceCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T: Sync + Send> Sync for OnceCell<T> {}
unsafe impl<T: Send> Send for OnceCell<T> {}

/// A value that is computed on first access.
///
/// The initializer `F` is stored inline, so no heap allocation is needed. The
/// value is computed once (lazily) and then cached for the lifetime of the
/// `Lazy`.
pub struct Lazy<T, F = fn() -> T> {
    cell: OnceCell<T>,
    init: UnsafeCell<Option<F>>,
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    /// Create a new `Lazy` with the given initializer.
    pub const fn new(f: F) -> Self {
        Lazy {
            cell: OnceCell::new(),
            init: UnsafeCell::new(Some(f)),
        }
    }

    /// Force initialization (if needed) and return a reference to the value.
    pub fn force(this: &Self) -> &T {
        this.cell.get_or_init(|| {
            // Safety: the initializer is taken exactly once during the
            // OnceCell's single initialization.
            unsafe { (*this.init.get()).take() }.expect("Lazy initializer missing")()
        })
    }
}

impl<T, F: FnOnce() -> T> core::ops::Deref for Lazy<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        Lazy::force(self)
    }
}

unsafe impl<T: Sync + Send, F: Send> Sync for Lazy<T, F> {}
unsafe impl<T: Send, F: Send> Send for Lazy<T, F> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn once_runs_once() {
        let once = Once::new();
        let mut count = 0;
        for _ in 0..10 {
            once.call_once(|| count += 1);
        }
        assert_eq!(count, 1);
        assert!(once.is_completed());
    }

    #[test]
    fn once_cell_init() {
        let cell: OnceCell<i32> = OnceCell::new();
        assert_eq!(cell.get(), None);
        assert_eq!(*cell.get_or_init(|| 42), 42);
        assert_eq!(*cell.get_or_init(|| 99), 42);
        assert_eq!(cell.get(), Some(&42));
    }

    #[test]
    fn lazy_value() {
        let lazy: Lazy<i32, _> = Lazy::new(|| 7 * 6);
        assert_eq!(*lazy, 42);
        assert_eq!(*Lazy::force(&lazy), 42);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// OnceCell initialized to a value always reports that value.
        #[test]
        fn cell_holds_value(v in any::<i64>()) {
            let cell: OnceCell<i64> = OnceCell::new();
            prop_assert_eq!(*cell.get_or_init(|| v), v);
            prop_assert_eq!(cell.get(), Some(&v));
        }

        /// Lazy deref equals the initializer's output.
        #[test]
        fn lazy_holds_value(v in any::<i64>()) {
            let lazy: Lazy<i64, _> = Lazy::new(move || v);
            prop_assert_eq!(*lazy, v);
        }
    }
}
