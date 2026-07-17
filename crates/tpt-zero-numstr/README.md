# tpt-zero-numstr

Integer and float to-string conversion and back, for `#![no_std]`
environments. Zero external dependencies; an `alloc` feature adds
`String`-returning convenience wrappers.

- `format_int` / `parse_int` — signed and unsigned integers of any width,
  with explicit radix control (2..=36), writing into a caller-provided buffer.
- `format_float` / `parse_float` — `f32`/`f64` formatting and parsing,
  including decimal and scientific (`1.5e3`) notation, plus `NaN`/`inf`.

## Example

```rust
use tpt_zero_numstr::{format_int, parse_int, format_float, parse_float};

let mut buf = [0u8; 32];
let s = core::str::from_utf8(format_int(12345i32, 10, &mut buf).unwrap()).unwrap();
assert_eq!(s, "12345");
assert_eq!(parse_int::<i32>(s.as_bytes(), 10), Some(12345));

let mut fbuf = [0u8; 32];
let f = core::str::from_utf8(format_float(3.14f64, &mut fbuf).unwrap()).unwrap();
assert_eq!(f, "3.14");
assert_eq!(parse_float::<f64>(f.as_bytes()), Some(3.14));
```

With the `alloc` feature:

```rust
# #[cfg(feature = "alloc")]
# {
use tpt_zero_numstr::{format_int_to_string, format_float_to_string};
assert_eq!(format_int_to_string(42i32, 10).unwrap(), "42");
assert_eq!(format_float_to_string(3.14f64).unwrap(), "3.14");
# }
```

## Float limitation

Float formatting uses the standard `core::fmt` rendering and parsing uses a
manual decimal reader. This is **not** guaranteed to be the shortest
round-trippable decimal representation for every value; for most inputs the
output round-trips exactly through `parse_float`, but do not depend on
bit-exact shortest-repr fidelity, especially for very large magnitudes or
17-digit shortest decimals (the manual `10^e` scaling can lose low-order
bits).

## `no_std`

`#![no_std]` with **zero** external dependencies. The `alloc` feature only
adds `String`-returning wrappers.

## License

Licensed under MIT or Apache-2.0 at your option.
