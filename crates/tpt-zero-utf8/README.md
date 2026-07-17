# tpt-zero-utf8

Zero-dependency, `#![no_std]` UTF-8 utilities for `no_std` and bare-metal
environments.

- Validate byte slices as UTF-8 ([`from_bytes`]).
- Find the nearest char-start byte before/after an arbitrary index
  ([`next_char_boundary`], [`prev_char_boundary`]).
- Iterate Unicode scalars safely without panicking on invalid input
  ([`Utf8Str::char_indices`]).
- Encode a `char` into a fixed buffer ([`encode_char`]).

## Example

```rust
use tpt_zero_utf8::{from_bytes, next_char_boundary};

let bytes = "héllo 世界".as_bytes();
let s = from_bytes(bytes).unwrap();
assert_eq!(s.as_str(), "héllo 世界");

// Slice at a byte offset without splitting a code point.
let cut = next_char_boundary(bytes, 3);
assert!(from_bytes(&bytes[..cut]).is_ok());
```

### Iterating scalars without panicking

Invalid bytes are surfaced as sentinel chars in the range `0xDC00..=0xDCFF`
rather than panicking or silently inserting replacement characters:

```rust
use tpt_zero_utf8::Utf8Str;

let s = Utf8Str::from_bytes_unchecked(&[0xFF, b'a']);
let mut it = s.char_indices();
let (_, bad) = it.next().unwrap();
assert!((bad as u32) >= 0xDC00);
```

## `no_std`

This crate is `#![no_std]` with **zero** external dependencies. It depends only
on `core`.

## License

Licensed under MIT or Apache-2.0 at your option.
