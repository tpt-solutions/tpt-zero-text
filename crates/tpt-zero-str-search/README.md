# tpt-zero-str-search

Zero-dependency, `#![no_std]` substring search.

- `find_byte` / `rfind_byte` — memchr-style single-byte search.
- `find` / `rfind` — Boyer-Moore-Horspool substring search over byte slices.
- `Finder` — a reusable searcher that precomputes its bad-character table once.

## Example

```rust
use tpt_zero_str_search::{find, Finder};

assert_eq!(find(b"ana", b"banana"), Some(1));

let finder = Finder::new(b"lazy");
assert_eq!(finder.find_in(b"the quick brown fox is lazy"), Some(23));
```

Needles are limited to 64 bytes (the bad-character table is a fixed 256-entry
array plus an inline 64-byte needle copy). For typical keys/tokens this is more
than enough; longer needles fall back to a simple linear scan via [`Finder`]'s
comparison loop.

## `no_std`

This crate is `#![no_std]` with **zero** external dependencies.

## License

Licensed under MIT or Apache-2.0 at your option.
