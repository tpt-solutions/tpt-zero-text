# tpt-zero-vec

Minimal `f32` vector math for `#![no_std]`, with zero dependencies.

- [`Vec2`], [`Vec3`], [`Vec4`] — `#[repr(C)]` `f32` tuples.
- Full `core::ops` overloads (`+`, `-`, `*`, `/` with scalars and same-type).
- `dot`, `cross` (Vec3), `length`, `length_sq`, `normalize` (via
  [`tpt-zero-fast-math`]).
- `min`/`max`, and `Vec3`/`Vec4` conversions (`xy`, `xyz`, `extend`).

## Example

```rust
use tpt_zero_vec::Vec3;

let a = Vec3::new(1.0, 0.0, 0.0);
let b = Vec3::new(0.0, 1.0, 0.0);
assert_eq!(a.dot(b), 0.0);
assert_eq!(a.cross(b), Vec3::new(0.0, 0.0, 1.0));

let dir = Vec3::new(0.0, 3.0, 0.0).normalize();
assert!((dir.length() - 1.0).abs() < 1e-3);
```

## Scope

`f32`-only in v0.1; `f64` vectors are deferred to v0.2. Length/normalization
use a fast square-root approximation (`~1e-3` error). There is no `alloc` API
in v0.1.

## `no_std`

`#![no_std]` with zero external dependencies (beyond `tpt-zero-fast-math`).

## License

Licensed under MIT or Apache-2.0 at your option.
