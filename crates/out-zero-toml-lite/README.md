# out-zero-toml-lite

> **Not published to crates.io.** The `toml` crate itself now supports `#![no_std]`, and
> `toml_parser`/`boml` already fill the lean/no-alloc niche this crate targeted. Kept here
> as an internal/reference implementation.

A minimal TOML parser for `#![no_std]`, with zero dependencies.

- `#![no_std]` core: [`parse_into`] fills a caller buffer with [`Entry`]
  `(section, key, value)` tuples. Value bytes are borrowed from the input.
- `Value`: `String` / `Integer` / `Float` / `Boolean` (raw tokens borrowed).
- `as_int` / `as_float` helpers (via `tpt-zero-numstr`).
- `alloc`: [`parse`] returns an owned [`Document`].

## Example

```rust
use out_zero_toml_lite::{parse_into, Entry, Value};

let input = b"[server]\nport = 8080\nname = \"main\"\n";
let mut out = [Entry::placeholder(); 8];
let n = parse_into(input, &mut out).unwrap();
assert_eq!(n, 2);
assert_eq!(out[0].section, Some(&b"server"[..]));
assert_eq!(out[0].key, b"port");
assert_eq!(out[0].value, Value::Integer(b"8080"));
```

## Scope (`-lite`, v0.1)

Single-level `[section]` tables and top-level `key = value` only. Excludes
arrays-of-tables, multi-line strings, datetime types, inline tables, and
dotted keys. A `[a.b]` header is treated as a literal section named `a.b`.
Strings are basic (`"..."`) with `\"`, `\\`, `\n`, `\t` escapes recognized at
the lexical level (escape decoding of the inner text is left to the caller).

## `no_std`

`#![no_std]` core (parser) with zero external dependencies (beyond
`tpt-zero-utf8`, `tpt-zero-numstr`, `tpt-zero-str-search`).

## License

Licensed under MIT or Apache-2.0 at your option.
