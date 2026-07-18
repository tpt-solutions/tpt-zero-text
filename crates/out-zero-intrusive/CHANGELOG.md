# Changelog

All notable changes to this crate will be documented in this file.

## 0.1.0

- Initial release.
- `Link<T>` embedded-link type with `is_linked`.
- `IntrusiveNode` trait mapping a node to its embedded link.
- `List<T>` intrusive doubly-linked list: `push_front` / `push_back` /
  `pop_front` / `pop_back`, `iter`, `len`, `is_empty`.
- `Drop` detaches every link so nodes can be re-inserted safely.
- Unsafe API with documented invariants and `debug_assert!` guards.
