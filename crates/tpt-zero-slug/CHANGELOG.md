# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-17

### Added
- `slugify_into` and `SlugWriter`: `#![no_std]` buffer-writer slug generation
  with Latin-1 transliteration.
- `slugify` (alloc feature): heap-allocated `String` wrapper.
- Curated Latin-1-Supplement transliteration table (`é`→`e`, `ß`→`ss`, …).
