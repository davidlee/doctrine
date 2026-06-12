# just check gates workspace members — cordage suite now run (SL-052 closed the hole)

`justfile` `test:` runs `cargo test --workspace` (not bare `cargo test`). The repo
root is both a package and the workspace root, so bare `cargo test` exercised only
the root package and SKIPPED every member — `crates/cordage` (incl.
`tests/denylist.rs`, the REQ-079 product-neutrality boundary) never ran under
`just check`, and a cordage-only regression landed green. SL-052 widened the recipe
to `--workspace`, so all members are now gated; any future workspace member is
auto-gated by the same recipe.

**Supersedes** `mem.pattern.build.just-check-tests-root-package-only` — its
described gate hole (bare `cargo test` skips members; ISS-007 sits red) is CLOSED
as of SL-052. Treat that memory as historical, not current state.

Workspace today = `.` + `crates/cordage` only (`Cargo.toml` members). The lone slow
suite (~28s) is cordage's debug-build scale test, now part of every `just check`.

Still live footgun when probing cordage's denylist manually: it bakes
`CARGO_MANIFEST_DIR` at compile time — a stale test binary panics "root resolution
is wrong (0 files)" and MASKS a real vocabulary hit. Force a recompile
(`touch crates/cordage/src/lib.rs`) before trusting a pass/fail. See
`mem.pattern.testing.stale-cargo-bin-exe`.
