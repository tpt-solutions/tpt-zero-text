# tpt-zero-text

The Structural Layer of the `tpt-zero-*` ecosystem: robust, zero-external-dependency
data structures, text processing, and mathematical primitives that scale from `core`
to `alloc` to `std` via feature flags.

Every crate in this workspace is `#![no_std]` by default (a `core`-only build with
zero external production dependencies) and gains `String`/`Vec`-returning convenience
APIs behind an opt-in `alloc` feature. Intra-workspace crates may depend on each other
(e.g. `tpt-zero-matrix` depends on `tpt-zero-vec`), but nothing in this workspace ever
pulls in a dependency from outside the `tpt-zero-*` family.

Target audience: WASM developers, game developers, IoT application writers, and
developers building configuration-driven `no_std` systems.

## Crates

| Tier | Crate | Purpose |
|------|-------|---------|
| 0 | [tpt-zero-utf8](crates/tpt-zero-utf8) | UTF-8 validation, char-boundary finding, safe scalar iteration |
| 0 | [tpt-zero-numstr](crates/tpt-zero-numstr) | Integer/float to string conversion and back |
| 0 | [tpt-zero-str-search](crates/tpt-zero-str-search) | Fast byte/substring search algorithms |
| 0 | [tpt-zero-fast-math](crates/tpt-zero-fast-math) | Approximate sqrt/sin/cos/tan via lookup tables |
| 0 | [tpt-zero-arrayvec](crates/tpt-zero-arrayvec) | `Vec`-like structure backed by a fixed-size array |
| 0 | [tpt-zero-ring](crates/tpt-zero-ring) | Fixed-capacity circular buffer |
| 0 | [tpt-zero-intrusive](crates/tpt-zero-intrusive) | Intrusive doubly-linked lists |
| 0 | [tpt-zero-once](crates/tpt-zero-once) | `no_std` lazy initialization / run-once logic |
| 0 | [tpt-zero-spin](crates/tpt-zero-spin) | Spinlock-based mutex/rwlock for `no_std` |
| 0 | [tpt-zero-channel](crates/tpt-zero-channel) | Lock-free bounded MPSC queue |
| 0 | [tpt-zero-color](crates/tpt-zero-color) | RGB/RGBA/HSV conversions and hex parsing |
| 1 | [tpt-zero-arraystring](crates/tpt-zero-arraystring) | `String`-like structure backed by a fixed-size array |
| 1 | [tpt-zero-linear-map](crates/tpt-zero-linear-map) | Sorted-array-backed map for small datasets |
| 1 | [tpt-zero-pool](crates/tpt-zero-pool) | Object pool to reuse allocations |
| 1 | [tpt-zero-slug](crates/tpt-zero-slug) | URL-safe string slug normalization |
| 1 | [tpt-zero-url-encode](crates/tpt-zero-url-encode) | Percent-encoding and decoding |
| 1 | [tpt-zero-xml-escape](crates/tpt-zero-xml-escape) | XML/HTML entity escaping and unescaping |
| 1 | [tpt-zero-glob](crates/tpt-zero-glob) | Glob pattern matching |
| 1 | [tpt-zero-json](crates/tpt-zero-json) | Pull-based, zero-allocation JSON tokenizer/parser |
| 1 | [tpt-zero-csv](crates/tpt-zero-csv) | Iterator-based, RFC 4180 compliant CSV parsing |
| 1 | [tpt-zero-toml-lite](crates/tpt-zero-toml-lite) | Restricted, safe TOML parser |
| 1 | [tpt-zero-ini](crates/tpt-zero-ini) | Simple INI configuration file parsing |
| 1 | [tpt-zero-vec](crates/tpt-zero-vec) | 2D/3D/4D vector math with operator overloading |
| 2 | [tpt-zero-matrix](crates/tpt-zero-matrix) | 3x3 and 4x4 matrix transformations |
| 2 | [tpt-zero-quat](crates/tpt-zero-quat) | Quaternions for 3D rotation |

See [TODO.md](TODO.md) for build progress and [PUBLISHING.md](PUBLISHING.md) for the
crates.io release runbook.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this workspace by you shall be dual licensed as above, without
any additional terms or conditions.
