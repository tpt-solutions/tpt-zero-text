# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-18

### Added
- `Reader`: `#![no_std]` RFC 4180 CSV reader with borrowed/buffered fields.
- `Field` (Borrowed / Buffered / Empty) and `CsvError`.
- `alloc`: `read_records` (owned `Vec<String>` rows) and `Writer`.
