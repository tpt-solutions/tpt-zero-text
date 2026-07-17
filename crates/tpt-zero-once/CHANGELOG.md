# Changelog

All notable changes to this crate will be documented in this file.

## 0.1.0

- Initial release.
- `Once` one-time initialization flag with spin-wait on contention.
- `OnceCell<T>` single-assignment cell returning stable references.
- `Lazy<T, F>` lazily-computed value with inline initializer (no `alloc`).
- `Acquire`/`Release` atomic ordering on the internal state machine.
