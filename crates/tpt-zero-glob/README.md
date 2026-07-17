# tpt-zero-glob

A small glob pattern matcher for `#![no_std]` environments, with zero
dependencies.

- `*` — match any run of characters within a single path segment.
- `?` — match exactly one character.
- `**` — match across path separators (zero or more segments).
- `[...]` — character class, with ranges (`a-z`) and negation (`[!...]`).

`{a,b}` alternation is **deferred to v0.2**.

## Example

```rust
use tpt_zero_glob::matches_str;

assert!(matches_str("src/*.rs", "src/lib.rs"));
assert!(matches_str("**/Cargo.toml", "a/b/Cargo.toml"));
assert!(matches_str("[a-z]?.txt", "ab.txt"));
assert!(!matches_str("src/*.rs", "src/a/b.rs"));
```

Compile once and reuse with [`Pattern`]:

```rust
use tpt_zero_glob::Pattern;

let p = Pattern::compile_str("build/**/*.o");
assert!(p.matches_str("build/obj/foo.o"));
```

## `no_std`

`#![no_std]` with zero external dependencies (beyond `tpt-zero-str-search`).

## License

Licensed under MIT or Apache-2.0 at your option.
