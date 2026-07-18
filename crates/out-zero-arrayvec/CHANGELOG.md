# Changelog

All notable changes to this crate will be documented in this file.

## 0.1.0

- Initial release.
- `ArrayVec<T, N>` on `[MaybeUninit<T>; N]` with sound `Drop`.
- `push` / `pop` / `insert` / `remove` / `truncate` / `clear`.
- `Deref<[T]>`, `iter` / `iter_mut`, `as_slice` / `as_slice_mut`.
- `CapacityError` on overflow instead of panicking.
