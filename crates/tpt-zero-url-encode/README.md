# tpt-zero-url-encode

RFC 3986 percent-encoding and decoding, for `#![no_std]` environments.

- `percent_encode_into` / `percent_decode_into` write into a caller-owned
  buffer (no allocation). The unreserved set `A-Z a-z 0-9 - . _ ~` passes
  through; everything else becomes `%XX`.
- `percent_encode` / `percent_decode` (enable the `alloc` feature) return owned
  `String`s.

Input is treated as raw bytes, so a UTF-8 string is encoded byte-by-byte
(each non-ASCII byte becomes one or more `%XX` sequences) — the conventional
behavior for URL component/query encoding. Decoding is non-lossy: an invalid
`%` sequence is copied through literally.

## Example

```rust
use tpt_zero_url_encode::percent_encode_into;

let mut buf = [0u8; 32];
let enc = percent_encode_into(b"a b&c", &mut buf).unwrap();
assert_eq!(enc, b"a%20b%26c");
```

## `no_std`

`#![no_std]` with **zero** external dependencies (built on `tpt-zero-utf8`).

## License

Licensed under MIT or Apache-2.0 at your option.
