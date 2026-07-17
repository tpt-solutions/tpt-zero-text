# tpt-zero-json

A small JSON tokenizer and value model for `#![no_std]`, with zero
dependencies.

- `Tokenizer` — a streaming, allocation-free, pull-based lexer. Strings with
  no escapes are returned by reference ([`Token::StringRaw`]); strings with
  escapes are decoded lazily as [`Token::StringChunk`]s through a small
  internal buffer. The tokenizer never allocates.
- `parse_number` — decode a number token's bytes. Integers parse exactly
  (via `tpt-zero-numstr`); everything else goes through the float reader.
- `JsonError` — a precise, offset-bearing error type.

Behind the `alloc` feature:

- [`Value`] — a JSON value tree. Object entries use `Vec<(String, Value)>`
  to **preserve insertion order** (not a hash map).
- [`from_slice`] / [`to_string`] — full parse and serialize.

## Example

```rust
use tpt_zero_json::Tokenizer;

let mut t = Tokenizer::new(br#"{"name":"tpt","n":42}"#);
let toks: Vec<_> = core::iter::from_fn(|| match t.next_token() {
    Ok(tpt_zero_json::Token::Eof) => None,
    Ok(tok) => Some(tok),
    Err(_) => None,
})
.collect();
assert!(!toks.is_empty());
```

With the `alloc` feature:

```rust
use tpt_zero_json::{from_slice, to_string, Value};

let v = from_slice(br#"{"a":1,"b":[true,false,null]}"#).unwrap();
assert_eq!(v.get("a").and_then(|x| x.as_f64()), Some(1.0));
let s = to_string(&v);
let v2 = from_slice(s.as_bytes()).unwrap();
assert_eq!(v, v2);
```

## Scope (v0.1)

The `alloc` layer does not enforce a maximum nesting depth, size limits, or
number canonicalization — it is intended for small, trusted-ish documents.
Duplicate object keys are all retained (last-wins is up to the caller).
Lone UTF-16 surrogates in `\u` escapes are passed through as-is.

## `no_std`

`#![no_std]` core (tokenizer) with zero external dependencies (beyond
`tpt-zero-utf8`, `tpt-zero-numstr`, `tpt-zero-str-search`).

## License

Licensed under MIT or Apache-2.0 at your option.
