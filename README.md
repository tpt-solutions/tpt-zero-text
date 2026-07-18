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

Every crate was reviewed against the existing crates.io ecosystem before release. Crates
that would just be a redundant clone of an established `no_std` incumbent (e.g. `arrayvec`,
`spin`, `percent-encoding`, `csv-core`) were kept in the repo as internal/reference
implementations but renamed from the `tpt-zero-` prefix to `out-zero-` and marked
`publish = false`, so publish intent is visible at a glance from the crate name alone. See
[TODO.md](TODO.md) for the full per-crate rationale.

### Published (`tpt-zero-*`)

| Tier | Crate | Purpose |
|------|-------|---------|
| 0 | [tpt-zero-utf8](crates/tpt-zero-utf8) | UTF-8 validation, char-boundary finding, safe scalar iteration |
| 0 | [tpt-zero-numstr](crates/tpt-zero-numstr) | Integer/float to string conversion and back |
| 0 | [tpt-zero-str-search](crates/tpt-zero-str-search) | Fast byte/substring search algorithms |
| 0 | [tpt-zero-fast-math](crates/tpt-zero-fast-math) | Approximate sqrt/sin/cos/tan via lookup tables |
| 0 | [tpt-zero-channel](crates/tpt-zero-channel) | Lock-free bounded MPSC queue |
| 0 | [tpt-zero-color](crates/tpt-zero-color) | RGB/RGBA/HSV conversions and hex parsing |
| 1 | [tpt-zero-slug](crates/tpt-zero-slug) | URL-safe string slug normalization |
| 1 | [tpt-zero-glob](crates/tpt-zero-glob) | Glob pattern matching |
| 1 | [tpt-zero-json](crates/tpt-zero-json) | Pull-based, zero-allocation JSON tokenizer/parser |
| 1 | [tpt-zero-vec](crates/tpt-zero-vec) | 2D/3D/4D vector math with operator overloading |
| 2 | [tpt-zero-matrix](crates/tpt-zero-matrix) | 3x3 and 4x4 matrix transformations |
| 2 | [tpt-zero-quat](crates/tpt-zero-quat) | Quaternions for 3D rotation |

### Internal / not published (`out-zero-*`)

Fully implemented and tested, but `publish = false` — kept for internal use and as a
reference implementation. Each crate's README names the crates.io incumbent it was judged
redundant with.

| Tier | Crate | Purpose |
|------|-------|---------|
| 0 | [out-zero-arrayvec](crates/out-zero-arrayvec) | `Vec`-like structure backed by a fixed-size array |
| 0 | [out-zero-ring](crates/out-zero-ring) | Fixed-capacity circular buffer |
| 0 | [out-zero-intrusive](crates/out-zero-intrusive) | Intrusive doubly-linked lists |
| 0 | [out-zero-once](crates/out-zero-once) | `no_std` lazy initialization / run-once logic |
| 0 | [out-zero-spin](crates/out-zero-spin) | Spinlock-based mutex/rwlock for `no_std` |
| 1 | [out-zero-arraystring](crates/out-zero-arraystring) | `String`-like structure backed by a fixed-size array |
| 1 | [out-zero-linear-map](crates/out-zero-linear-map) | Sorted-array-backed map for small datasets |
| 1 | [out-zero-pool](crates/out-zero-pool) | Object pool to reuse allocations |
| 1 | [out-zero-url-encode](crates/out-zero-url-encode) | Percent-encoding and decoding |
| 1 | [out-zero-xml-escape](crates/out-zero-xml-escape) | XML/HTML entity escaping and unescaping |
| 1 | [out-zero-csv](crates/out-zero-csv) | Iterator-based, RFC 4180 compliant CSV parsing |
| 1 | [out-zero-toml-lite](crates/out-zero-toml-lite) | Restricted, safe TOML parser |
| 1 | [out-zero-ini](crates/out-zero-ini) | Simple INI configuration file parsing |

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
