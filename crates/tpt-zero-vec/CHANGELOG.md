# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-18

### Added
- `Vec2`/`Vec3`/`Vec4` `f32` vector types with `core::ops` overloads.
- `dot`, `cross` (Vec3), `length`, `length_sq`, `normalize`, `min`, `max`.
- `Vec3`/`Vec4` conversions (`xy`, `xyz`, `extend`).
- Fast `sqrt`/`inv_sqrt` backing via `tpt-zero-fast-math`.
