# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-17

### Added
- `ArrayString<const N>`: a `String`-like buffer backed by a fixed-size array.
- UTF-8-safe `push`/`push_str` (reject overflow / split code points via
  `Utf8Error`), `pop`, `clear`, `Deref<str>`, and `core::fmt::Write`.
