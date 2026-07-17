# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-18

### Added
- `#![no_std]` INI parser: `[section]` headers and `key = value` / `key : value`
  assignments, with `;`/`#` comments.
- `Entry` tuples (borrowed) and `parse_into` (caller-buffered).
- `alloc`: `parse` returning an owned `Document`.
