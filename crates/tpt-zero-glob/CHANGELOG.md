# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-17

### Added
- `Pattern` with `compile`/`compile_str`, `matches`/`matches_str`.
- Wildcards `*` (in-segment), `?`, `**` (cross-segment), and `[...]` classes
  (ranges + negation).
- One-shot `matches` / `matches_str` helpers.
- `{a,b}` alternation deferred to v0.2.
