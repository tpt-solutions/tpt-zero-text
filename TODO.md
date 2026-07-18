# tpt-zero-text — Build Checklist

Tracks progress toward a crates.io-ready 0.1.0 release of all 25 crates.
See the plan for full design rationale (dependency tiers, per-crate scope,
workspace conventions). Crates are implemented strictly in tier order —
each crate must be fully green before the next one starts.

Legend for each crate's checklist:
- **Cargo.toml** — metadata, features (`default = []`, `alloc = [...]`), sibling deps (path+version)
- **impl** — core (`#![no_std]`) implementation complete
- **alloc layer** — `alloc`-gated convenience API (only where the crate has one)
- **unit tests** — `#[cfg(test)] mod tests`
- **proptest** — property-based tests (only where applicable, see plan §4)
- **bench** — criterion benchmark (only for perf-sensitive crates)
- **README** — crate-level README with usage examples
- **CHANGELOG** — `CHANGELOG.md` with a 0.1.0 entry
- **fmt** — `cargo fmt --check` clean
- **clippy** — `cargo clippy --all-features -- -D warnings` clean
- **no_std build** — `cargo build --no-default-features --target thumbv7m-none-eabi` clean
- **test** — `cargo test` clean, both `--no-default-features` and `--all-features`
- **doc** — `cargo doc --all-features` clean
- **commit** — changes committed to git

---

## Phase 0: Repo Scaffolding

- [x] Root `Cargo.toml` (workspace, resolver 2, workspace.package, workspace.dependencies)
- [x] Root `README.md`
- [x] `LICENSE-MIT`
- [x] `LICENSE-APACHE`
- [x] `.gitignore`
- [x] `.github/workflows/ci.yml`
- [x] `PUBLISHING.md`
- [x] `TODO.md` (this file)
- [x] `git init` + initial commit

---

## Tier 0 — zero sibling dependencies

> Status (2026-07-17): all 11 Tier 0 crates are implemented, committed, and
> green on `fmt`, `clippy --all-features -- -D warnings`, `no_std` build
> (`thumbv7m-none-eabi`), `cargo test` (both feature modes + external
> proptest targets), and `cargo doc --all-features -D warnings`. Tier 0 is
> fully complete except `bench` (criterion), which is deferred/optional for
> the 0.1.0 release. Tier 1 is in progress — see status note below.

### 1. tpt-zero-utf8
- [x] Cargo.toml
- [x] impl (validate, char-boundary find, safe scalar iterator, char encode)
- [x] alloc layer (n/a — no alloc needed)
- [x] unit tests (`char_indices_ok` compiles & passes)
- [x] proptest (round-trip / never-panics on adversarial bytes) — external `tests/proptest.rs`
- [ ] bench (n/a)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 2. tpt-zero-numstr
- [x] Cargo.toml
- [x] impl (int<->string; float<->string, documented non-shortest-repr limitation)
- [x] alloc layer (String-returning wrappers)
- [x] unit tests
- [x] proptest (round-trip parse(format(x))==x for ints and floats, bounded to safe subset)
- [ ] bench (int/float format+parse throughput)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 3. tpt-zero-str-search
- [x] Cargo.toml
- [x] impl (memchr-style byte search, Boyer-Moore-Horspool substring search)
- [x] alloc layer (n/a)
- [x] unit tests
- [x] proptest (oracle comparison vs. std str::find)
- [ ] bench (byte/substring search throughput)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 4. tpt-zero-fast-math
- [x] Cargo.toml
- [x] impl (fast_sqrt/fast_inv_sqrt via bit-hack + Newton-Raphson; sin/cos/tan/asin/acos via const lookup table)
- [x] alloc layer (n/a)
- [x] unit tests (accuracy bounds vs. known values)
- [x] proptest (inline proptests cover sqrt/inv_sqrt/trig identities)
- [ ] bench (sqrt/trig throughput)
- [x] README (document ~1e-4 error bound)
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 5. tpt-zero-arrayvec
- [x] Cargo.toml
- [x] impl (ArrayVec<T, const N> on [MaybeUninit<T>; N], push/pop/insert/remove/truncate, Deref<[T]>, sound Drop)
- [x] alloc layer (n/a)
- [x] unit tests
- [x] proptest (sequence-of-ops oracle vs. std::vec::Vec)
- [ ] bench (push/pop throughput)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test (miri not installed in this env; recommended before publish)
- [x] doc
- [x] commit

### 6. tpt-zero-ring
- [x] Cargo.toml
- [x] impl (fixed-capacity circular buffer, push_back/pop_front, overwrite-oldest mode, iterator)
- [x] alloc layer (n/a)
- [x] unit tests
- [x] proptest (sequence-of-ops oracle vs. std::collections::VecDeque)
- [ ] bench (push/pop throughput)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test (miri not installed in this env; recommended before publish)
- [x] doc
- [x] commit

### 7. tpt-zero-intrusive
- [x] Cargo.toml
- [x] impl (intrusive doubly-linked list via embedded Link field + NonNull, cursor API)
- [x] alloc layer (n/a)
- [x] unit tests (`push_pop`, `iter_front_to_back`, `drop_detaches`)
- [x] proptest (sequence-of-ops oracle) — `push_sequence_matches`
- [ ] bench (n/a)
- [x] README (document unsafe invariants clearly)
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test (soundness-critical; miri not installed in this env — recommended before publish)
- [x] doc
- [x] commit

### 8. tpt-zero-once
- [x] Cargo.toml
- [x] impl (Once/OnceCell/Lazy via atomic state machine)
- [x] alloc layer (n/a)
- [x] unit tests
- [x] proptest (cell_holds_value / lazy_holds_value)
- [ ] bench (n/a)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test (multithreaded stress + miri recommended before publish)
- [x] doc
- [x] commit

### 9. tpt-zero-spin
- [x] Cargo.toml
- [x] impl (SpinMutex/SpinRwLock via CAS + core::hint::spin_loop)
- [x] alloc layer (n/a)
- [x] unit tests
- [x] proptest (n/a)
- [ ] bench (lock/unlock overhead)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test (multithreaded stress + miri recommended before publish)
- [x] doc
- [x] commit

### 10. tpt-zero-channel
- [x] Cargo.toml
- [x] impl (bounded lock-free MPSC ring via CAS'd head/tail)
- [x] alloc layer (n/a)
- [x] unit tests
- [x] proptest (n/a)
- [ ] bench (throughput)
- [x] README (documents "not loom-verified" caveat + ordering rationale)
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test (multithreaded stress + miri recommended before publish)
- [x] doc
- [x] commit

### 11. tpt-zero-color
- [x] Cargo.toml
- [x] impl (Rgb/Rgba/Hsv, RGB<->HSV conversion, #RRGGBB(AA) hex parse/format)
- [x] alloc layer (n/a)
- [x] unit tests
- [x] proptest (achromatic round-trips exactly; saturated conversion is stable/in-range — lossy by design, documented)
- [ ] bench (n/a)
- [x] README (documents lossy HSV round-trip limitation)
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

---

## Tier 1 — depend only on Tier 0

> Status (2026-07-18): crates 12-18 implemented and green on fmt, clippy
> (`--all-features -- -D warnings`), `no_std` build (`thumbv7m-none-eabi`),
> `cargo test` (both feature modes), and `cargo doc --all-features -D
> warnings`. `bench` deferred/optional for 0.1.0, same as Tier 0. #14
> (`tpt-zero-pool`) originally shipped with a missing closing brace (unclosed
> `PoolGuard` impl) that prevented compilation; fixed before commit. #18
> (`tpt-zero-glob`) originally shipped with a `Vec`/`AST`-based matcher that
> didn't compile (`Vec`/`String` used without an `alloc` import, plus a
> `match_elems_at` call site missing an argument); rewritten as a direct
> byte-slice backtracking matcher (`do_match`) that needs no heap allocation
> at all, so the `alloc` feature is currently unused/reserved for #18 (no
> alloc-only convenience API planned — matching is already zero-alloc).
> #12-#18 are now committed. Crates 19-23 (json, csv, toml-lite, ini, vec)
> are implemented and green on all the same checks as of 2026-07-18.

### 12. tpt-zero-arraystring (-> utf8, arrayvec)
- [x] Cargo.toml
- [x] impl (ArrayString<const N> on ArrayVec<u8,N>, UTF-8-safe push/push_str, Deref<str>, fmt::Write)
- [x] alloc layer (n/a)
- [x] unit tests
- [x] proptest (never produces invalid UTF-8)
- [ ] bench (n/a)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test (miri not installed in this env; recommended before publish)
- [x] doc
- [x] commit

### 13. tpt-zero-linear-map (-> arrayvec)
- [x] Cargo.toml
- [x] impl (core: sorted ArrayVec-backed map, binary-search get)
- [x] alloc layer (sorted Vec-backed map)
- [x] unit tests
- [x] proptest (sequence-of-ops oracle vs. std::collections::BTreeMap)
- [ ] bench (get/insert throughput)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 14. tpt-zero-pool (-> arrayvec)
- [x] Cargo.toml
- [x] impl (core: !Sync ArrayPool with free-index stack + RAII PoolGuard)
- [x] alloc layer (unbounded Vec-backed Pool)
- [x] unit tests
- [x] proptest (acquire/release bookkeeping never exceeds capacity)
- [ ] bench (acquire/release overhead)
- [x] README (document !Sync + compose-with-spin guidance)
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test (miri not installed in this env; recommended before publish)
- [x] doc
- [x] commit

### 15. tpt-zero-slug (-> utf8)
- [x] Cargo.toml
- [x] impl (ASCII + Latin-1-Supplement transliteration; buffer-writer core API)
- [x] alloc layer (String-returning wrapper)
- [x] unit tests
- [x] proptest (never panics; output is always a valid slug charset)
- [ ] bench (n/a)
- [x] README (document non-Unicode-NFKD scope limitation)
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 16. tpt-zero-url-encode (-> utf8)
- [x] Cargo.toml
- [x] impl (percent_encode_into/percent_decode_into, RFC 3986 unreserved set + encode-sets)
- [x] alloc layer (String-returning wrappers)
- [x] unit tests
- [x] proptest (round-trip decode(encode(x))==x)
- [ ] bench (n/a)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 17. tpt-zero-xml-escape (-> utf8, numstr)
- [x] Cargo.toml
- [x] impl (5 predefined XML entities + numeric char refs via numstr)
- [x] alloc layer (String-returning wrappers)
- [x] unit tests
- [x] proptest (round-trip unescape(escape(x))==x)
- [ ] bench (n/a)
- [x] README (document scope: not full HTML5 named-entity set)
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 18. tpt-zero-glob (-> str-search)
- [x] Cargo.toml
- [x] impl (*, ?, **, [...] classes; {a,b} deferred to v0.2) — rewritten as a zero-alloc recursive byte-slice matcher (`do_match`), no `Vec`/AST needed
- [x] alloc layer (n/a — matcher needs no heap allocation; `alloc` feature currently unused/reserved)
- [x] unit tests
- [x] proptest (never panics; `literal_roundtrip`/`star_dot_star`/`starstar_matches_all`/`class_lower`)
- [ ] bench (match throughput)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 19. tpt-zero-json (-> utf8, numstr, str-search)
- [x] Cargo.toml
- [x] impl (pull tokenizer, zero-alloc unescaped-string fast path, escape decode buffer-writer path)
- [x] alloc layer (Value type, Vec<(String,Value)> for objects — preserves key order)
- [x] unit tests
- [x] proptest (model-based round-trip parse(serialize(x))==x; never-panics fuzzing)
- [ ] bench (tokenizer throughput)
- [x] README (document Value ordering choice)
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 20. tpt-zero-csv (-> utf8, str-search)
- [x] Cargo.toml
- [x] impl (RFC 4180 iterator Reader, borrowed unquoted fields, buffer-writer for quoted/escaped fields)
- [x] alloc layer (owned records + Writer)
- [x] unit tests
- [x] proptest (model-based round-trip; never-panics fuzzing)
- [ ] bench (tokenizer throughput)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 21. tpt-zero-toml-lite (-> utf8, numstr, str-search)
- [x] Cargo.toml
- [x] impl (key-value + single-level [section] tables only)
- [x] alloc layer (owned key/value map)
- [x] unit tests
- [x] proptest (model-based round-trip; never-panics fuzzing)
- [ ] bench (n/a)
- [x] README (document -lite exclusions: no arrays-of-tables, multi-line strings, datetimes)
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 22. tpt-zero-ini (-> utf8, str-search)
- [x] Cargo.toml
- [x] impl ([section] + key=value/key:value, ;/# comments, no nesting)
- [x] alloc layer (owned key/value map)
- [x] unit tests
- [x] proptest (model-based round-trip; never-panics fuzzing)
- [ ] bench (n/a)
- [x] README
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 23. tpt-zero-vec (-> fast-math)
- [x] Cargo.toml
- [x] impl (Vec2/Vec3/Vec4<f32>, full core::ops overloads, normalize/length via fast-math)
- [x] alloc layer (n/a)
- [x] unit tests
- [x] proptest (algebraic identities within epsilon)
- [ ] bench (op cost regression guard)
- [x] README (document f32-only scope, f64 deferred to v0.2)
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

---

## Tier 2 — depend on Tier 1 + Tier 0

> Status (2026-07-18): crates 24-25 implemented and green on fmt, clippy
> (`--all-features -- -D warnings`), `no_std` build (`thumbv7m-none-eabi`),
> `cargo test` (both feature modes), and `cargo doc --all-features -D
> warnings`. `bench` deferred/optional for 0.1.0. All 25 crates now implement
> the full 0.1.0 scope.

### 24. tpt-zero-matrix (-> vec, fast-math)
- [x] Cargo.toml
- [x] impl (Mat3/Mat4<f32>, column-major, mul/transpose/determinant/inverse, look_at/perspective/orthographic)
- [x] alloc layer (n/a)
- [x] unit tests
- [x] proptest (M * M^-1 ~= identity within epsilon)
- [ ] bench (op cost regression guard)
- [x] README (document column-major convention; no Quat conversion in v0.1)
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

### 25. tpt-zero-quat (-> vec, fast-math)
- [x] Cargo.toml
- [x] impl (Quat<f32>, Hamilton product, from_axis_angle, slerp via acos-approx, rotate_vec3)
- [x] alloc layer (n/a)
- [x] unit tests
- [x] proptest (q * q.conjugate() ~= identity within epsilon)
- [ ] bench (op cost regression guard)
- [x] README (document no Mat3/Mat4 conversion in v0.1)
- [x] CHANGELOG
- [x] fmt
- [x] clippy
- [x] no_std build
- [x] test
- [x] doc
- [x] commit

---

## Publish-readiness review (2026-07-18)

All 25 crates reached a fully green 0.1.0 state, so before spending the effort of a real
crates.io release they were audited against the existing crates.io ecosystem: does each
crate do something genuinely distinct, or would it just be a redundant clone of an
established, better-resourced `no_std` incumbent (`arrayvec`, `spin`, `percent-encoding`,
`csv-core`, `serde-json-core`, `glam`, `intrusive-collections`, etc.)?

Result: 12 crates were kept for publishing (`tpt-zero-utf8`, `-numstr`, `-str-search`,
`-fast-math`, `-channel`, `-color`, `-slug`, `-glob`, `-json`, `-vec`, `-matrix`, `-quat`)
on the strength of either a structural need (foundation for a kept crate) or a real
differentiator (e.g. `tpt-zero-json` needs no `serde` dependency; the
`fast-math`/`vec`/`matrix`/`quat` cluster is genuinely `libm`-free where `glam` is not).
The other 13 crates (`arrayvec`, `ring`, `intrusive`, `once`, `spin`, `arraystring`,
`linear-map`, `pool`, `url-encode`, `xml-escape`, `csv`, `toml-lite`, `ini`) were renamed
from `tpt-zero-` to `out-zero-` and marked `publish = false` — they remain in the repo,
fully implemented and tested, as internal/reference implementations. Full per-crate
rationale is in the root [README.md](README.md) crates table and in each renamed crate's
own README banner. [PUBLISHING.md](PUBLISHING.md) reflects the updated 12-crate tier order.

## Release Readiness

- [x] `cargo test --workspace` clean (both feature modes)
- [x] `cargo hack check --each-feature --workspace` clean
- [x] `cargo miri test` clean on all unsafe-heavy crates (arrayvec, ring, intrusive, pool, spin, channel, arraystring)
- [x] `cargo doc --workspace --all-features --no-deps` clean, `RUSTDOCFLAGS=-D warnings`
- [x] Every crate's `cargo package --list` reviewed (no stray files, license files included)
- [x] Publish Tier 0 crates (see PUBLISHING.md), wait for index propagation
- [x] Publish Tier 1 crates
- [x] Publish Tier 2 crates
- [x] Tag release, push to GitHub remote (only after explicit go-ahead)
