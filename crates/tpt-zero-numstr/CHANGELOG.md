# Changelog

All notable changes to this crate will be documented in this file.

## 0.1.0

- Initial release.
- `format_int` / `parse_int` for all signed and unsigned integer widths with
  radix control (2..=36).
- `format_float` / `parse_float` for `f32`/`f64` with decimal, scientific, and
  `NaN`/`inf` handling.
- `alloc` feature with `String`-returning `format_int_to_string` /
  `format_float_to_string` wrappers.
- Documented non-shortest-repr limitation for float round-tripping.
