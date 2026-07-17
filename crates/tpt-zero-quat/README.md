# tpt-zero-quat

Minimal `f32` quaternion math for `#![no_std]`, with zero dependencies.

- [`Quat`] stored as `(x, y, z, w)` with `w` the scalar part.
- Hamilton product (`mul`), `conjugate`, `invert`.
- `from_axis_angle` construction; `rotate_vec3` vector rotation.
- `slerp` spherical-linear interpolation.
- `f32`-only; trig/square-root back through `tpt-zero-fast-math`.

## Example

```rust
use tpt_zero_quat::Quat;
use tpt_zero_vec::Vec3;

let q = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), core::f32::consts::FRAC_PI_2);
let v = Vec3::new(1.0, 0.0, 0.0);
let rotated = q.rotate_vec3(v);
```

## Scope (v0.1)

`f32`-only. No `Mat3`/`Mat4` conversion in v0.1 (see `tpt-zero-matrix`).
Quaternions are assumed approximately unit-length for `rotate_vec3`; they are
normalized internally. Expect `~1e-3` accuracy from the fast-math backing.

## `no_std`

`#![no_std]` with zero external dependencies (beyond `tpt-zero-vec`,
`tpt-zero-fast-math`).

## License

Licensed under MIT or Apache-2.0 at your option.
