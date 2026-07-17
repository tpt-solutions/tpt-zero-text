# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-18

### Added
- `Tokenizer`: `#![no_std]` streaming JSON lexer.
- `Token::StringRaw` zero-copy fast path and `Token::StringChunk` lazy escape
  decoding (with `\uXXXX` support).
- `JsonError` with byte-offset context.
- `parse_number` (integer-exact via `tpt-zero-numstr`, float otherwise).
- `alloc`: `Value` (insertion-ordered objects), `from_slice`, `to_string`.
