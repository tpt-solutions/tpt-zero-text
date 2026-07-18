# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-17

### Added
- `escape_into` / `unescape_into`: `#![no_std]` buffer-writer XML entity codec.
- Five predefined XML entities plus decimal/hex numeric character references
  (via `tpt-zero-numstr`).
- `escape` / `unescape` (alloc feature): `String`-returning wrappers.
