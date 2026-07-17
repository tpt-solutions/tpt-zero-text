# tpt-zero-pool

A fixed-capacity object pool with RAII guards, for `#![no_std]` environments
where you want to reuse a bounded set of slots instead of allocating.

- `ArrayPool<T, N>` holds `N` slots inline. `acquire(value)` returns a
  [`PoolGuard`] that owns a slot for its lifetime and returns it (dropping the
  value) on drop. When the pool is full, `acquire` returns `None`.
- `Pool<T>` (enable the `alloc` feature) is an unbounded `Vec`-backed pool.

## `!Sync` by design

`ArrayPool` and `Pool` manage internal slot bookkeeping without locking, so
they are **not `Sync`**. To share a pool across threads, wrap it in a
`tpt-zero-spin` `SpinMutex`:

```rust
use tpt_zero_pool::ArrayPool;
use tpt_zero_spin::SpinMutex;

let pool = SpinMutex::new(ArrayPool::<u32, 8>::new());
let mut guard = pool.lock();
let slot = guard.acquire(42).unwrap();
assert_eq!(*slot, 42);
```

## `no_std`

`#![no_std]` with **zero** external dependencies.

## License

Licensed under MIT or Apache-2.0 at your option.
