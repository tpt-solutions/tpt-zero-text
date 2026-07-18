# Publishing Runbook

Crates in this workspace depend on each other, so they must be published to
crates.io in strict tier order. Within a tier, order does not matter — there
are no intra-tier edges by construction.

Before publishing anything:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo test --workspace --no-default-features && cargo test --workspace --all-features`
4. `cargo doc --workspace --all-features --no-deps` (with `RUSTDOCFLAGS=-D warnings`)
5. CI green on the commit being published.

Only the 12 crates below are published; the other 13 crates in `crates/` carry the
`out-zero-` prefix and `publish = false` — they were judged redundant with an existing
crates.io incumbent and are kept as internal/reference implementations only (see the
root [README.md](README.md) and each crate's own README for the rationale). Do not add
them to the lists below.

## Tier 0 (publish first, any order within the tier)

```
tpt-zero-utf8
tpt-zero-numstr
tpt-zero-str-search
tpt-zero-fast-math
tpt-zero-channel
tpt-zero-color
```

## Tier 1 (publish after every Tier 0 crate is live on crates.io)

```
tpt-zero-slug
tpt-zero-glob
tpt-zero-json
tpt-zero-vec
```

## Tier 2 (publish after every Tier 1 crate is live on crates.io)

```
tpt-zero-matrix
tpt-zero-quat
```

## Per-crate publish steps

```sh
cd crates/<name>
cargo package --list      # sanity-check the file list (no stray files; LICENSE-* included)
cargo publish --dry-run   # only fully meaningful once this crate's path deps are live upstream
cargo publish
```

Path dependencies on siblings must always carry both `path =` and `version =`
so they resolve correctly once published (crates.io ignores `path`, resolves
by `version` only).

After publishing a Tier 0 crate, wait for crates.io index propagation
(usually well under a minute, but `cargo publish --dry-run` for a dependent
crate can transiently fail immediately after — retry after a short wait)
before starting the next tier.
