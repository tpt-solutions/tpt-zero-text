# tpt-zero-ini

A minimal INI parser for `#![no_std]`, with zero dependencies.

- `#![no_std]` core: [`parse_into`] fills a caller buffer with [`Entry`]
  `(section, key, value)` tuples. Bytes are borrowed from the input.
- `key = value` and `key : value` both supported.
- `;` and `#` begin comments (stripped to end of line).
- `alloc`: [`parse`] returns an owned [`Document`].

## Example

```rust
use tpt_zero_ini::{parse_into, Entry};

let input = b"[server]\nport = 8080\nname = main\n";
let mut out = [Entry::placeholder(); 8];
let n = parse_into(input, &mut out).unwrap();
assert_eq!(n, 2);
assert_eq!(out[0].section, Some(&b"server"[..]));
assert_eq!(out[0].key, b"port");
assert_eq!(out[0].value, b"8080");
```

## Scope (v0.1)

No nesting, multi-line values, or escaping. Each assignment is its own entry;
duplicate keys are all retained (last-wins is up to the caller). A later
`[section]` simply switches the current section.

## `no_std`

`#![no_std]` core (parser) with zero external dependencies (beyond
`tpt-zero-utf8`, `tpt-zero-str-search`).

## License

Licensed under MIT or Apache-2.0 at your option.
