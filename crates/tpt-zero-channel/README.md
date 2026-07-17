# tpt-zero-channel

A bounded, lock-free multi-producer / single-consumer (MPSC) channel for
`#![no_std]`, with **zero** external dependencies.

- Fixed-capacity ring buffer; producers enqueue without locking (CAS'd
  head/tail) so many producers can send concurrently.
- A single consumer drains with [`Receiver::recv`].
- Back-pressure is explicit: a full ring returns [`TrySendError::Full`]; an
  empty ring returns [`TryRecvError::Empty`].

```rust
use tpt_zero_channel::{Channel, channel, TrySendError};

let ch: Channel<i32, 4> = Channel::new();
let (tx, rx) = channel(&ch);
assert!(tx.send(42).is_ok());
assert_eq!(rx.recv(), Ok(42));
```

## Ordering caveat

This is a best-effort lock-free design using `AcqRel`/`Acquire`/`Release`
atomics. It is **not** verified with `loom` or `miri` in this release. Validate
under your own stress tests before relying on it in production.

## `no_std`

`#![no_std]` with **zero** external dependencies.

## License

Licensed under MIT or Apache-2.0 at your option.
