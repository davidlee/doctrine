# just gate runs the --workspace test gate; just check is the fast root-only inner loop

The justfile splits two gates (the key slug predates the split — the alias is not
authoritative; read this body):

- **`just check`** = `fmt lint test build`, where `test:` runs **bare `cargo test`**
  (root package only). Fast inner-loop variant. Because the repo root is both a
  package and the workspace root, bare `cargo test` exercises only the root package
  and SKIPS every member — `crates/cordage` (incl. `tests/denylist.rs`, the REQ-079
  product-neutrality boundary) does NOT run under `check`.
- **`just gate`** = `fmt lint test-all build`, where `test-all:` runs `cargo test
  --workspace`. This is the end-of-phase / CI gate that exercises every workspace
  member, so cordage is gated here, not in `check`.

History: a cordage-only regression once landed green because `check` ran bare
`cargo test`. SL-052 widened `check` itself to `--workspace`; a later chore reverted
`check` to fast root-only and moved the `--workspace` guarantee to the new `gate`
recipe — to keep the inner loop fast while preserving the full gate. **The
pre-commit / end-of-phase guard is now `just gate`, not `just check`** (AGENTS.md
updated). Any future workspace member is auto-gated by `gate`'s `--workspace`.

**Supersedes** `mem.pattern.build.just-check-tests-root-package-only` — its
described gate hole (bare `cargo test` skips members; ISS-007 sits red) is CLOSED
as of SL-052. Treat that memory as historical, not current state.

Workspace today = `.` + `crates/cordage` only (`Cargo.toml` members). The lone slow
suite (~28s) is cordage's debug-build scale test — the reason `gate` is slow and
`check` stays fast.

Still live footgun when probing cordage's denylist manually: it bakes
`CARGO_MANIFEST_DIR` at compile time — a stale test binary panics "root resolution
is wrong (0 files)" and MASKS a real vocabulary hit. Force a recompile
(`touch crates/cordage/src/lib.rs`) before trusting a pass/fail. See
`mem.pattern.testing.stale-cargo-bin-exe`.
