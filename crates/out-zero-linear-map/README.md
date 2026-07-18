# out-zero-linear-map

> **Not published to crates.io.** Overlaps the [`linear-map`](https://crates.io/crates/linear-map)
> crate and `heapless::LinearMap`. Kept here as an internal/reference implementation.

A small ordered map backed by a sorted sequence, for `#![no_std]`
environments.

- Elements are kept **sorted by key**, so lookups use binary search (`O(log n)`)
  and iteration yields keys in order.
- Insertion/removal are `O(n)` (elements are shifted); ideal for small datasets
  where heap overhead dominates.
- [`ArrayLinearMap<K, V, N>`] — fixed-capacity, `#![no_std]`, backed by
  `out-zero-arrayvec`.
- [`VecLinearMap<K, V>`] (enable the `alloc` feature) — unbounded, backed by
  `alloc::vec::Vec`.

## Example

```rust
use out_zero_linear_map::ArrayLinearMap;

let mut m: ArrayLinearMap<&str, i32, 8> = ArrayLinearMap::new();
m.insert("b", 2).ok();
m.insert("a", 1).ok();
assert_eq!(m.get(&"a"), Some(&1));
// Iteration is in key order: "a" then "b".
assert_eq!(m.keys().copied().next(), Some("a"));
```

## `no_std`

`#![no_std]` with **zero** external dependencies. Enable the `alloc` feature
for the unbounded `VecLinearMap`.

## License

Licensed under MIT or Apache-2.0 at your option.
