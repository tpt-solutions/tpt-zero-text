# tpt-zero-fast-math

Zero-dependency, `#![no_std]` approximate math for games, DSP, and
embedded workloads.

- `fast_sqrt` / `fast_inv_sqrt` — IEEE-754 bit trick + one Newton-Raphson step.
- `sin` / `cos` / `tan` / `asin` / `acos` — const lookup table with linear interpolation.
- `isqrt` — exact integer square root (no floats).

## Accuracy

These are **approximations**. Typical error is ~`1e-4` relative for
`sqrt`/`inv_sqrt` and ~`1e-4` radians for the trig functions. Do not
use them where bit-exact `std` math is required.

## Example

```rust
use tpt_zero_fast_math::{fast_sqrt, sin, cos};

assert!((fast_sqrt(4.0) - 2.0).abs() < 1e-3);
let theta = 0.5;
// sin^2 + cos^2 ~= 1
assert!((sin(theta) * sin(theta) + cos(theta) * cos(theta) - 1.0).abs() < 1e-3);
```

## `no_std`

This crate is `#![no_std]` with **zero** external dependencies.

## License

Licensed under MIT or Apache-2.0 at your option.
