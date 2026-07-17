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
- [ ] `git init` + initial commit

---

## Tier 0 — zero sibling dependencies

### 1. tpt-zero-utf8
- [ ] Cargo.toml
- [ ] impl (validate, char-boundary find, safe scalar iterator, char encode)
- [ ] alloc layer (n/a — no alloc needed)
- [ ] unit tests
- [ ] proptest (round-trip / never-panics on adversarial bytes)
- [ ] bench (n/a)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 2. tpt-zero-numstr
- [ ] Cargo.toml
- [ ] impl (int<->string; float<->string, documented non-shortest-repr limitation)
- [ ] alloc layer (String-returning wrappers)
- [ ] unit tests
- [ ] proptest (round-trip parse(format(x))==x for ints and floats)
- [ ] bench (int/float format+parse throughput)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 3. tpt-zero-str-search
- [ ] Cargo.toml
- [ ] impl (memchr-style byte search, Boyer-Moore-Horspool substring search)
- [ ] alloc layer (n/a)
- [ ] unit tests
- [ ] proptest (oracle comparison vs. std str::find)
- [ ] bench (byte/substring search throughput)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 4. tpt-zero-fast-math
- [ ] Cargo.toml
- [ ] impl (fast_sqrt/fast_inv_sqrt via bit-hack + Newton-Raphson; sin/cos/tan/asin/acos via const lookup table)
- [ ] alloc layer (n/a)
- [ ] unit tests (accuracy bounds vs. known values)
- [ ] proptest (n/a — deterministic table-driven, covered by unit tests)
- [ ] bench (sqrt/trig throughput)
- [ ] README (document ~1e-4 error bound)
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 5. tpt-zero-arrayvec
- [ ] Cargo.toml
- [ ] impl (ArrayVec<T, const N> on [MaybeUninit<T>; N], push/pop/insert/remove/truncate, Deref<[T]>, sound Drop)
- [ ] alloc layer (n/a)
- [ ] unit tests
- [ ] proptest (sequence-of-ops oracle vs. std::vec::Vec)
- [ ] bench (push/pop throughput)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test (+ miri)
- [ ] doc
- [ ] commit

### 6. tpt-zero-ring
- [ ] Cargo.toml
- [ ] impl (fixed-capacity circular buffer, push_back/pop_front, overwrite-oldest mode, iterator)
- [ ] alloc layer (n/a)
- [ ] unit tests
- [ ] proptest (sequence-of-ops oracle vs. std::collections::VecDeque)
- [ ] bench (push/pop throughput)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test (+ miri)
- [ ] doc
- [ ] commit

### 7. tpt-zero-intrusive
- [ ] Cargo.toml
- [ ] impl (intrusive doubly-linked list via embedded Link field + NonNull, cursor API)
- [ ] alloc layer (n/a)
- [ ] unit tests
- [ ] proptest (sequence-of-ops oracle)
- [ ] bench (n/a)
- [ ] README (document unsafe invariants clearly)
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test (+ miri — soundness-critical)
- [ ] doc
- [ ] commit

### 8. tpt-zero-once
- [ ] Cargo.toml
- [ ] impl (Once/OnceCell/Lazy via atomic state machine)
- [ ] alloc layer (n/a)
- [ ] unit tests
- [ ] proptest (n/a)
- [ ] bench (n/a)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test (+ multithreaded stress test, + miri)
- [ ] doc
- [ ] commit

### 9. tpt-zero-spin
- [ ] Cargo.toml
- [ ] impl (SpinMutex/SpinRwLock via CAS + core::hint::spin_loop)
- [ ] alloc layer (n/a)
- [ ] unit tests
- [ ] proptest (n/a)
- [ ] bench (lock/unlock overhead)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test (+ multithreaded stress test, + miri)
- [ ] doc
- [ ] commit

### 10. tpt-zero-channel
- [ ] Cargo.toml
- [ ] impl (bounded lock-free MPSC ring via CAS'd head/tail)
- [ ] alloc layer (n/a)
- [ ] unit tests
- [ ] proptest (n/a)
- [ ] bench (throughput)
- [ ] README (document "not loom-verified" caveat + ordering rationale)
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test (+ multithreaded stress test, + miri)
- [ ] doc
- [ ] commit

### 11. tpt-zero-color
- [ ] Cargo.toml
- [ ] impl (Rgb/Rgba/Hsv, RGB<->HSV conversion, #RRGGBB(AA) hex parse/format)
- [ ] alloc layer (n/a)
- [ ] unit tests
- [ ] proptest (round-trip RGB->HSV->RGB within epsilon)
- [ ] bench (n/a)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

---

## Tier 1 — depend only on Tier 0

### 12. tpt-zero-arraystring (-> utf8, arrayvec)
- [ ] Cargo.toml
- [ ] impl (ArrayString<const N> on ArrayVec<u8,N>, UTF-8-safe push/push_str, Deref<str>, fmt::Write)
- [ ] alloc layer (n/a)
- [ ] unit tests
- [ ] proptest (never produces invalid UTF-8)
- [ ] bench (n/a)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test (+ miri)
- [ ] doc
- [ ] commit

### 13. tpt-zero-linear-map (-> arrayvec)
- [ ] Cargo.toml
- [ ] impl (core: sorted ArrayVec-backed map, binary-search get)
- [ ] alloc layer (sorted Vec-backed map)
- [ ] unit tests
- [ ] proptest (sequence-of-ops oracle vs. std::collections::BTreeMap)
- [ ] bench (get/insert throughput)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 14. tpt-zero-pool (-> arrayvec)
- [ ] Cargo.toml
- [ ] impl (core: !Sync ArrayPool with free-index stack + RAII PoolGuard)
- [ ] alloc layer (unbounded Vec-backed Pool)
- [ ] unit tests
- [ ] proptest (n/a)
- [ ] bench (acquire/release overhead)
- [ ] README (document !Sync + compose-with-spin guidance)
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test (+ miri)
- [ ] doc
- [ ] commit

### 15. tpt-zero-slug (-> utf8)
- [ ] Cargo.toml
- [ ] impl (ASCII + Latin-1-Supplement transliteration; buffer-writer core API)
- [ ] alloc layer (String-returning wrapper)
- [ ] unit tests
- [ ] proptest (never panics; output is always a valid slug charset)
- [ ] bench (n/a)
- [ ] README (document non-Unicode-NFKD scope limitation)
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 16. tpt-zero-url-encode (-> utf8)
- [ ] Cargo.toml
- [ ] impl (percent_encode_into/percent_decode_into, RFC 3986 unreserved set + encode-sets)
- [ ] alloc layer (String-returning wrappers)
- [ ] unit tests
- [ ] proptest (round-trip decode(encode(x))==x)
- [ ] bench (n/a)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 17. tpt-zero-xml-escape (-> utf8, numstr)
- [ ] Cargo.toml
- [ ] impl (5 predefined XML entities + numeric char refs via numstr)
- [ ] alloc layer (String-returning wrappers)
- [ ] unit tests
- [ ] proptest (round-trip unescape(escape(x))==x)
- [ ] bench (n/a)
- [ ] README (document scope: not full HTML5 named-entity set)
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 18. tpt-zero-glob (-> str-search)
- [ ] Cargo.toml
- [ ] impl (*, ?, **, [...] classes; {a,b} deferred to v0.2)
- [ ] alloc layer (n/a, or Vec<Match> collection under alloc)
- [ ] unit tests
- [ ] proptest (oracle comparison vs. reference matcher)
- [ ] bench (match throughput)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 19. tpt-zero-json (-> utf8, numstr, str-search)
- [ ] Cargo.toml
- [ ] impl (pull tokenizer, zero-alloc unescaped-string fast path, escape decode buffer-writer path)
- [ ] alloc layer (Value type, Vec<(String,Value)> for objects — preserves key order)
- [ ] unit tests
- [ ] proptest (model-based round-trip parse(serialize(x))==x; never-panics fuzzing)
- [ ] bench (tokenizer throughput)
- [ ] README (document Value ordering choice)
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 20. tpt-zero-csv (-> utf8, str-search)
- [ ] Cargo.toml
- [ ] impl (RFC 4180 iterator Reader, borrowed unquoted fields, buffer-writer for quoted/escaped fields)
- [ ] alloc layer (owned records + Writer)
- [ ] unit tests
- [ ] proptest (model-based round-trip; never-panics fuzzing)
- [ ] bench (tokenizer throughput)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 21. tpt-zero-toml-lite (-> utf8, numstr, str-search)
- [ ] Cargo.toml
- [ ] impl (key-value + single-level [section] tables only)
- [ ] alloc layer (owned key/value map)
- [ ] unit tests
- [ ] proptest (model-based round-trip; never-panics fuzzing)
- [ ] bench (n/a)
- [ ] README (document -lite exclusions: no arrays-of-tables, multi-line strings, datetimes)
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 22. tpt-zero-ini (-> utf8, str-search)
- [ ] Cargo.toml
- [ ] impl ([section] + key=value/key:value, ;/# comments, no nesting)
- [ ] alloc layer (owned key/value map)
- [ ] unit tests
- [ ] proptest (model-based round-trip; never-panics fuzzing)
- [ ] bench (n/a)
- [ ] README
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 23. tpt-zero-vec (-> fast-math)
- [ ] Cargo.toml
- [ ] impl (Vec2/Vec3/Vec4<f32>, full core::ops overloads, normalize/length via fast-math)
- [ ] alloc layer (n/a)
- [ ] unit tests
- [ ] proptest (algebraic identities within epsilon)
- [ ] bench (op cost regression guard)
- [ ] README (document f32-only scope, f64 deferred to v0.2)
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

---

## Tier 2 — depend on Tier 1 + Tier 0

### 24. tpt-zero-matrix (-> vec, fast-math)
- [ ] Cargo.toml
- [ ] impl (Mat3/Mat4<f32>, column-major, mul/transpose/determinant/inverse, look_at/perspective/orthographic)
- [ ] alloc layer (n/a)
- [ ] unit tests
- [ ] proptest (M * M^-1 ~= identity within epsilon)
- [ ] bench (op cost regression guard)
- [ ] README (document column-major convention; no Quat conversion in v0.1)
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

### 25. tpt-zero-quat (-> vec, fast-math)
- [ ] Cargo.toml
- [ ] impl (Quat<f32>, Hamilton product, from_axis_angle, slerp via acos-approx, rotate_vec3)
- [ ] alloc layer (n/a)
- [ ] unit tests
- [ ] proptest (q * q.conjugate() ~= identity within epsilon)
- [ ] bench (op cost regression guard)
- [ ] README (document no Mat3/Mat4 conversion in v0.1)
- [ ] CHANGELOG
- [ ] fmt
- [ ] clippy
- [ ] no_std build
- [ ] test
- [ ] doc
- [ ] commit

---

## Release Readiness

- [ ] `cargo test --workspace` clean (both feature modes)
- [ ] `cargo hack check --each-feature --workspace` clean
- [ ] `cargo miri test` clean on all unsafe-heavy crates (arrayvec, ring, intrusive, pool, spin, channel, arraystring)
- [ ] `cargo doc --workspace --all-features --no-deps` clean, `RUSTDOCFLAGS=-D warnings`
- [ ] Every crate's `cargo package --list` reviewed (no stray files, license files included)
- [ ] Publish Tier 0 crates (see PUBLISHING.md), wait for index propagation
- [ ] Publish Tier 1 crates
- [ ] Publish Tier 2 crates
- [ ] Tag release, push to GitHub remote (only after explicit go-ahead)
