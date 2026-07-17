# tpt-zero-matrix

Minimal `f32` matrix math for `#![no_std]`, with zero dependencies.

- [`Mat3`] / [`Mat4`] stored **column-major** (`columns[c][r]`).
- `mul`, `transpose`, `determinant`, `invert` (cofactor expansion),
  `transform`.
- View/projection builders: `look_at`, `perspective`, `orthographic`
  (right-handed). `from_row_major` for convenient construction.
- `f32`-only; length/normalize back through `tpt-zero-vec` /
  `tpt-zero-fast-math`.

## Example

```rust
use tpt_zero_matrix::Mat4;

let m = Mat4::identity();
let inv = m.invert().unwrap();
assert_eq!(m.mul(&inv), Mat4::identity());
```

## Scope (v0.1)

`f32`-only. No `Quat` conversion in v0.1 (see `tpt-zero-quat`). Inverse uses
cofactor expansion with a `1e-8` singularity threshold; expect `~1e-3`
accuracy from the fast-math backing.

## `no_std`

`#![no_std]` with zero external dependencies (beyond `tpt-zero-vec`,
`tpt-zero-fast-math`).

## License

Licensed under MIT or Apache-2.0 at your option.
