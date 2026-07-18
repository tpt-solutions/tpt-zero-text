# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-17

### Added
- `ArrayLinearMap<K, V, N>`: fixed-capacity ordered map backed by a sorted
  `ArrayVec`, with binary-search `get`/`get_mut`, `insert` (replace on dup),
  `remove`, and ordered iteration.
- `VecLinearMap<K, V>` (alloc feature): unbounded ordered map backed by `Vec`.
