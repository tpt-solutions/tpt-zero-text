# tpt-zero-ring

A fixed-capacity circular buffer for streaming data, in `#![no_std]` with zero
dependencies.

- `Ring<T, N>` holds up to `N` elements inline.
- `push_back` appends; in error mode it returns `RingFull` when full, or in
  `overwrite` mode (via `new_overwrite`) it drops the oldest element.
- `pop_front` removes the oldest element; `front` / `back` peek; `iter`
  traverses oldest-to-newest without moving data.
- `Drop` runs element destructors for all live elements (sound).

## Example

```rust
use tpt_zero_ring::{Ring, RingFull};

let mut r: Ring<i32, 2> = Ring::new();
r.push_back(1).unwrap();
r.push_back(2).unwrap();
assert_eq!(r.push_back(3), Err(RingFull));

let mut r: Ring<i32, 2> = Ring::new_overwrite();
r.push_back(1).unwrap();
r.push_back(2).unwrap();
r.push_back(3).unwrap(); // drops 1
assert_eq!(r.pop_front(), Some(2));
```

## `no_std`

`#![no_std]` with **zero** external dependencies.

## License

Licensed under MIT or Apache-2.0 at your option.
