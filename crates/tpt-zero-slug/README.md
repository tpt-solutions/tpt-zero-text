# tpt-zero-slug

ASCII/Latin-1-safe slug generation for `#![no_std]` environments, with zero
dependencies.

- Lowercase ASCII alphanumeric output separated by single hyphens.
- A curated set of Latin-1-Supplement characters transliterated to ASCII
  (`é`→`e`, `ß`→`ss`, `ü`→`ue`, …).
- Buffer-writer core API ([`slugify_into`], [`SlugWriter`]) — no heap required.

## Example

```rust
use tpt_zero_utf8::Utf8Str;
use tpt_zero_slug::slugify_into;

let input = Utf8Str::from_bytes_unchecked(b"The Quick Brown Fox!");
let mut buf = [0u8; 64];
let slug = slugify_into(input, &mut buf);
assert_eq!(core::str::from_utf8(slug).unwrap(), "the-quick-brown-fox");
```

With the `alloc` feature:

```rust
use tpt_zero_utf8::Utf8Str;
use tpt_zero_slug::slugify;

let input = Utf8Str::from_bytes_unchecked("Café Münchën".as_bytes());
assert_eq!(slugify(input), "cafe-munchen");
```

## Scope

This crate performs **lightweight** transliteration. It does **not** implement
full Unicode NFKD normalization (no combining-mark stripping, no multi-script
transliteration). Characters outside the covered Latin-1 range are dropped.
For full Unicode slugging, use a dedicated crate.

## `no_std`

`#![no_std]` with zero external dependencies (beyond `tpt-zero-utf8`, also
`no_std`). The core API allocates nothing.

## License

Licensed under MIT or Apache-2.0 at your option.
