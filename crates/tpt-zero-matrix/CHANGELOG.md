# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-18

### Added
- `Mat3`/`Mat4` (column-major) with `mul`, `transpose`, `determinant`,
  `invert`, `transform`.
- `look_at`, `perspective`, `orthographic` (right-handed) view/projection
  builders; `from_row_major` constructor.
- `fast-math` backing via `tpt-zero-vec` / `tpt-zero-fast-math`.
