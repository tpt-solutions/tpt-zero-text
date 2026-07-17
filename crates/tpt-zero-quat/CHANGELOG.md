# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-18

### Added
- `Quat` (`(x, y, z, w)`) with Hamilton product, `conjugate`, `invert`,
  `normalize`.
- `from_axis_angle` construction and `rotate_vec3` vector rotation.
- `slerp` spherical-linear interpolation.
- `fast-math` backing via `tpt-zero-vec` / `tpt-zero-fast-math`.
