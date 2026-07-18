# out-zero-spin

> **Not published to crates.io.** Superseded by the [`spin`](https://crates.io/crates/spin)
> crate, which covers the same design (spinlock mutex/RW-lock for `#![no_std]`) and is the
> de facto standard for this niche. Kept here as an internal/reference implementation.

Spin-based mutual-exclusion locks for `#![no_std]`, with **zero** external
dependencies.

- [`SpinMutex<T>`] — a mutual-exclusion lock. Acquire spins (via
  `core::hint::spin_loop`) until free; the RAII guard `Deref`s to the value.
- [`SpinRwLock<T>`] — a reader-writer lock: many concurrent readers or a single
  exclusive writer.

```rust
use out_zero_spin::SpinMutex;

let m = SpinMutex::new(0u32);
*m.lock() += 1;
assert_eq!(*m.lock(), 1);
```

## When to use

Spin locks busy-wait, so they suit only very short critical sections or
contexts where blocking is unavailable (e.g. interrupt handlers). Prefer an
OS-backed mutex when one is available.

> This release is **not** verified with `loom` or `miri`. Validate under your
> own concurrency tests before relying on it in production.

## `no_std`

`#![no_std]` with **zero** external dependencies.

## License

Licensed under MIT or Apache-2.0 at your option.
