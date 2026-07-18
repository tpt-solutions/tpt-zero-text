# Changelog

All notable changes to this crate will be documented in this file.

## 0.1.0

- Initial release.
- `SpinMutex<T>` with `Acquire`/`Release` CAS-based acquisition and RAII guard.
- `SpinRwLock<T>` supporting many concurrent readers or one exclusive writer.
- `AcqRel`/`Release` atomic ordering on the lock words.
