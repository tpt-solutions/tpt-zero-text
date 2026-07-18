# out-zero-once

> **Not published to crates.io.** Overlaps [`conquer-once`](https://crates.io/crates/conquer-once)
> (spin-lock-based `Once`/`OnceCell`/`Lazy` for `#![no_std]`) and `once_cell::race`. Kept
> here as an internal/reference implementation.

One-time initialization primitives for `#![no_std]`, with **zero** external
dependencies.

- [`Once`] — runs an initialization closure at most once; concurrent callers
  spin until the first completes.
- [`OnceCell<T>`] — holds a `T` initialized exactly once; `get_or_init` returns
  a stable reference.
- [`Lazy<T, F>`] — a value computed on first access. The initializer is stored
  inline, so no `alloc` is needed.

```rust
use out_zero_once::Lazy;

static CONFIG: Lazy<u32, fn() -> u32> = Lazy::new(|| 42);
assert_eq!(*CONFIG, 42);
```

## Ordering

The state machine uses `core::sync::atomic` `Acquire`/`Release` ordering so the
initialization write is visible to every caller that observes the "done" state.

> This release is **not** verified with `loom` or `miri`. Validate under your
> own concurrency tests before relying on it in production.

## `no_std`

`#![no_std]` with **zero** external dependencies.

## License

Licensed under MIT or Apache-2.0 at your option.
