# tpt-zero-arrayvec

A `Vec`-like collection backed by a fixed-size inline array, for `#![no_std]`
environments where heap allocation is unavailable.

- `ArrayVec<T, N>` stores up to `N` elements inline.
- `push` / `pop` / `insert` / `remove` / `truncate`, all capacity-checked
  (no panics; `push`/`insert` return `CapacityError`).
- Derefs to `&[T]`, with `iter` / `iter_mut` and `as_slice`.
- `Drop` runs element destructors only for the initialized prefix (sound).

## Example

```rust
use tpt_zero_arrayvec::ArrayVec;

let mut v: ArrayVec<i32, 4> = ArrayVec::new();
assert!(v.push(1).is_ok());
assert!(v.push(2).is_ok());
assert_eq!(&*v, &[1, 2]);
// Pushing past capacity is a safe error, not a panic:
assert!(v.push(3).is_ok());
assert!(v.push(4).is_ok());
assert_eq!(v.push(5), Err(tpt_zero_arrayvec::CapacityError));
```

## `no_std`

`#![no_std]` with **zero** external dependencies.

## License

Licensed under MIT or Apache-2.0 at your option.
