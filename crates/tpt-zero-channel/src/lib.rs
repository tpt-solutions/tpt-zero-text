#![no_std]
//! `tpt-zero-channel`: a bounded, lock-free multi-producer / single-consumer
//! (MPSC) channel for `#![no_std]`.
//!
//! The channel is a fixed-capacity ring buffer. Producers [`send`](Sender::send)
//! without locking (a CAS'd head/tail lets many producers enqueue
//! concurrently); a single consumer [`recv`](Receiver::recv)s. When the ring is
//! full, `send` returns [`TrySendError::Full`]; when empty, `recv` returns
//! [`TryRecvError::Empty`].
//!
//! # Capacity
//!
//! The ring capacity is `N - 1` slots; one slot is reserved to distinguish
//! "full" from "empty" without an extra atomic.
//!
//! # Ordering caveat
//!
//! This release is a best-effort lock-free design using `AcqRel`/`Acquire`/
//! `Release` atomics. It is **not** verified with `loom` or `miri`. The
//! intended ordering contract is sequentially-consistent publication of each
//! written slot before its index becomes visible to the consumer; validate
//! under your own stress tests before relying on it in production.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Error returned when [`Sender::send`] finds the channel full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrySendError {
    /// The channel is currently full; try again later.
    Full,
}

/// Error returned when [`Receiver::recv`] finds the channel empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryRecvError {
    /// The channel is currently empty; try again later.
    Empty,
}

struct Slot<T> {
    /// Written flag: `0` = empty, `1` = contains a value.
    ready: AtomicUsize,
    value: UnsafeCell<Option<T>>,
}

/// A bounded MPSC channel over `N` slots.
pub struct Channel<T, const N: usize> {
    head: AtomicUsize, // next slot producers write
    tail: AtomicUsize, // next slot consumer reads
    slots: [Slot<T>; N],
}

impl<T, const N: usize> Channel<T, N> {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT_SLOT: Slot<T> = Slot {
        ready: AtomicUsize::new(0),
        value: UnsafeCell::new(None),
    };

    /// Create a new, empty channel.
    pub const fn new() -> Self {
        Channel {
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            slots: [Self::INIT_SLOT; N],
        }
    }

    fn next(idx: usize) -> usize {
        (idx + 1) % N
    }
}

impl<T, const N: usize> Default for Channel<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// The producing end of a [`Channel`].
pub struct Sender<'a, T, const N: usize> {
    channel: &'a Channel<T, N>,
}

impl<'a, T, const N: usize> Sender<'a, T, N> {
    /// Attempt to send `value`. Returns [`TrySendError::Full`] if the ring is
    /// full.
    pub fn send(&self, value: T) -> Result<(), TrySendError> {
        loop {
            let head = self.channel.head.load(Ordering::Relaxed);
            let next_head = Channel::<T, N>::next(head);
            // The ring holds at most `N - 1` items; it is full when the next
            // write position has caught up to the consumer's tail.
            if next_head == self.channel.tail.load(Ordering::Acquire) {
                return Err(TrySendError::Full);
            }
            // Claim the slot by reserving head -> next_head.
            if self
                .channel
                .head
                .compare_exchange(head, next_head, Ordering::AcqRel, Ordering::Relaxed)
                .is_err()
            {
                // A producer raced us; retry from the top.
                continue;
            }
            // Publish the value, then mark ready so the consumer may take it.
            unsafe {
                *self.channel.slots[head].value.get() = Some(value);
            }
            self.channel.slots[head]
                .ready
                .store(1, Ordering::Release);
            return Ok(());
        }
    }
}

/// The consuming end of a [`Channel`].
pub struct Receiver<'a, T, const N: usize> {
    channel: &'a Channel<T, N>,
}

impl<'a, T, const N: usize> Receiver<'a, T, N> {
    /// Attempt to receive a value. Returns [`TryRecvError::Empty`] if none is
    /// available.
    pub fn recv(&self) -> Result<T, TryRecvError> {
        let tail = self.channel.tail.load(Ordering::Relaxed);
        if self.channel.slots[tail].ready.load(Ordering::Acquire) != 1 {
            return Err(TryRecvError::Empty);
        }
        // Take the value out.
        let value = unsafe { (*self.channel.slots[tail].value.get()).take() }
            .expect("ready slot must hold a value");
        self.channel.slots[tail].ready.store(0, Ordering::Release);
        self.channel
            .tail
            .store(Channel::<T, N>::next(tail), Ordering::Release);
        Ok(value)
    }

    /// Whether a value is immediately available.
    pub fn is_empty(&self) -> bool {
        let tail = self.channel.tail.load(Ordering::Relaxed);
        self.channel.slots[tail].ready.load(Ordering::Acquire) != 1
    }
}

/// Split a [`Channel`] into its [`Sender`] and [`Receiver`] halves.
pub fn channel<T, const N: usize>(ch: &Channel<T, N>) -> (Sender<'_, T, N>, Receiver<'_, T, N>) {
    (Sender { channel: ch }, Receiver { channel: ch })
}

unsafe impl<T: Send, const N: usize> Sync for Channel<T, N> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_recv() {
        let ch: Channel<i32, 4> = Channel::new();
        let (tx, rx) = channel(&ch);
        assert!(rx.is_empty());
        assert!(tx.send(1).is_ok());
        assert!(tx.send(2).is_ok());
        assert_eq!(rx.recv(), Ok(1));
        assert_eq!(rx.recv(), Ok(2));
        assert_eq!(rx.recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn full_then_drain() {
        let ch: Channel<i32, 3> = Channel::new(); // capacity 2
        let (tx, rx) = channel(&ch);
        assert!(tx.send(1).is_ok());
        assert!(tx.send(2).is_ok());
        assert_eq!(tx.send(3), Err(TrySendError::Full));
        assert_eq!(rx.recv(), Ok(1));
        assert!(tx.send(3).is_ok());
        assert_eq!(rx.recv(), Ok(2));
        assert_eq!(rx.recv(), Ok(3));
    }
}

#[cfg(test)]
mod proptests {
    extern crate alloc;
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Every accepted send is received exactly once, in FIFO order.
        #[test]
        fn fifo(values in proptest::collection::vec(any::<i32>(), 0..16)) {
            let ch: Channel<i32, 4> = Channel::new();
            let (tx, rx) = channel(&ch);
            let mut sent = alloc::vec::Vec::new();
            for &v in &values {
                if tx.send(v).is_ok() {
                    sent.push(v);
                }
            }
            // Drain whatever is in the channel.
            let mut got: alloc::vec::Vec<i32> = alloc::vec::Vec::new();
            while let Ok(v) = rx.recv() {
                got.push(v);
            }
            prop_assert_eq!(got, sent);
        }
    }
}
