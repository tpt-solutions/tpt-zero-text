# Changelog

All notable changes to this crate will be documented in this file.

## 0.1.0

- Initial release.
- `Ring<T, N>` fixed-capacity circular buffer with sound `Drop`.
- `push_back` (error mode + `new_overwrite` mode), `pop_front`, `front`/`back`.
- `iter` oldest-to-newest without moving data.
- `RingFull` error on overflow in error mode.
