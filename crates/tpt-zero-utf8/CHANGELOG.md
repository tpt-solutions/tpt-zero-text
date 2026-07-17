# Changelog

All notable changes to this crate will be documented in this file.

## 0.1.0

- Initial release.
- `from_bytes` UTF-8 validation with `ValidationError`.
- `next_char_boundary` / `prev_char_boundary` helpers.
- `Utf8Str` transparent wrapper with safe `CharIndices` iterator.
- `encode_char` scalar encoder into a fixed buffer.
