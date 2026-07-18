# out-zero-arraystring

> **Not published to crates.io.** Superseded by the
> [`arraystring`](https://crates.io/crates/arraystring) crate, which covers the same design
> (fixed-capacity inline string, opt-out-of-std). Kept here as an internal/reference
> implementation.

A `String`-like buffer backed by a fixed-size inline array, for `#![no_std]`
environments where heap allocation is unavailable.

- `ArrayString<const N>` stores up to `N` UTF-8 bytes inline (on top of
  `out-zero-arrayvec`).
- All pushes are UTF-8-safe: a `push`/`push_str` that would split a code point
  or overflow capacity is rejected with a `Utf8Error` rather than panicking.
- Derefs to `str`, implements `core::fmt::Write`.

## Example

```rust
use out_zero_arraystring::ArrayString;

let mut s: ArrayString<8> = ArrayString::new();
s.push_str("hi").unwrap();
s.push('!').unwrap();
assert_eq!(&*s, "hi!");

// Pushing past capacity (or a split code point) is a safe error:
let mut small: ArrayString<2> = ArrayString::new();
assert!(small.push_str("abc").is_err());
```

## `no_std`

`#![no_std]` with **zero** external dependencies (built on `tpt-zero-utf8`
and `out-zero-arrayvec`).

## License

Licensed under MIT or Apache-2.0 at your option.
