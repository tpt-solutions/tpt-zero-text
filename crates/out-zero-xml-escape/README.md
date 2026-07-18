# out-zero-xml-escape

> **Not published to crates.io.** Overlaps [`html-escape`](https://crates.io/crates/html-escape)
> (standalone, `#![no_std]`-optional) and `quick-xml`'s `escape` module. Kept here as an
> internal/reference implementation.

XML/HTML entity escaping and unescaping for `#![no_std]` environments, with
zero dependencies.

- Escapes the five predefined XML entities (`&` `<` `>` `"` `'`) via
  [`escape_into`].
- Unescapes those five plus numeric character references (`&#NN;` decimal and
  `&#xHH;` hex) via [`unescape_into`], using [`tpt-zero-numstr`] for the digits.
- Buffer-writer core API — no heap required. Alloc feature adds `String`
  wrappers.

## Example

```rust
use out_zero_xml_escape::escape_into;

let mut buf = [0u8; 64];
let n = escape_into(b"<a>&'\"", &mut buf).unwrap();
assert_eq!(&buf[..n], b"&lt;a&gt;&amp;&apos;&quot;");
```

With the `alloc` feature:

```rust
use out_zero_xml_escape::{escape, unescape};

assert_eq!(escape(b"<>&"), "&lt;&gt;&amp;");
assert_eq!(unescape(b"&lt;&gt;&amp;"), "<>&");
```

## Scope

This is **not** a full HTML5 named-entity decoder. Only the five predefined XML
entities and numeric references are supported; unknown named entities
(`&copy;`, `&euro;`, …) are passed through verbatim. This keeps the crate
`no_std`, dependency-free, and predictable.

## `no_std`

`#![no_std]` with zero external dependencies (beyond `tpt-zero-utf8` and
`tpt-zero-numstr`, also `no_std`).

## License

Licensed under MIT or Apache-2.0 at your option.
