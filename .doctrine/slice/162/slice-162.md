# Runtime-resolve test binary path

## Context

Integration tests spawn the built `doctrine` CLI via a compile-time constant:

```rust
const BIN: &str = env!("CARGO_BIN_EXE_doctrine");   // frozen absolute path
```

`env!` bakes one tree's absolute path (`<target>/debug/doctrine`) into the test
artifact at build time. The jail (`/workspace/...`) and the host (`/home/...`)
bind-mount the **same** working tree, so they share **one** in-tree `target/`
and therefore one compiled test artifact. Cargo's fingerprint does not treat the
baked path as an input, so it serves that artifact to whichever namespace runs
next. When the running namespace ≠ the namespace that last compiled, the baked
path does not exist there and the spawn fails:

```
spawn doctrine: Os { code: 2, kind: NotFound } (tests/…:NN)
```

Every `CARGO_BIN_EXE_doctrine` e2e suite fails as a block under this mismatch
(observed: `e2e_adr_cli_golden`, `e2e_backlog_filter_alias`). Rebuilding the
*bin* cannot fix it — the *test* is not recompiled (no fingerprint change), so
the stale baked path persists.

This is the **same footgun CHR-014 already closed** for `env!("CARGO_MANIFEST_DIR")`:
resolve at runtime, never at compile time. CHR-014 added `test_support::repo_root()`
(runtime resolver) + the `e2e_no_baked_manifest_dir` guard that bans the macro.
Its header scoped itself to "footgun #1 (path-baking)" but only treated the
manifest-dir macro; the sibling `CARGO_BIN_EXE_doctrine` path was left untreated
across 59 test files. This slice extends the blessed pattern to the bin path.

See [[mem.fact.testing.runtime-manifest-dir]]. Note the tension with
[[mem.fact.build.in-tree-per-worktree-target]] (claims no shared
`CARGO_TARGET_DIR`): the sharing here is cross-**namespace** (jail vs host mount
view of one tree), not cross-worktree — the runtime-resolve fix is correct under
either mechanism. `/design` to reconcile the precise sharing model.

## Scope & Objectives

- Add a runtime resolver `test_support::doctrine_bin() -> PathBuf` — derive the
  bin location from the running test exe (`current_exe()` → grandparent-sibling
  `<target>/<profile>/doctrine`), so it tracks the live namespace, profile, and
  target dir with no baked absolute path.
- Expose it through `tests/common/mod.rs` (the existing CHR-014 seam) alongside
  `repo_root`.
- Replace all 59 `const BIN: &str = env!("CARGO_BIN_EXE_doctrine")` sites with
  the runtime resolver at every `Command::new(BIN)` call site.
- Extend the `e2e_no_baked_manifest_dir` guard to also ban
  `env!("CARGO_BIN_EXE…")`, renaming/generalising it so the regression net covers
  both path-baking macros.

## Non-Goals

- IMP-004 / footgun #2 (cargo fingerprint serving a stale *artifact* across
  trees, mitigated by `just rebuild-stale`) — out of scope; this slice only
  removes the baked *path*, not stale-artifact reuse.
- Changing the jail's `CARGO_TARGET_DIR` / mount topology.
- Any change to the `doctrine` CLI itself or to golden expectations — the e2e
  goldens stay **byte-for-byte unchanged** (behaviour-preservation gate).

## Affected surface

- `src/test_support.rs` — new resolver (one source, shared by lib + integration).
- `tests/common/mod.rs` — re-export.
- `tests/e2e_*.rs` (59 files) + `tests/relation_cli.rs` — swap const → resolver.
- `tests/e2e_no_baked_manifest_dir.rs` — broaden the guard.

## Risks / Assumptions

- Layout assumption: `doctrine` sits at `<target>/<profile>/doctrine`, sibling of
  the test exe's `deps/` parent. True for cargo default + shared `CARGO_TARGET_DIR`.
- Loses the cargo guarantee that `CARGO_BIN_EXE_*` only resolves when the bin is a
  declared dep. Mitigation: keep running via `cargo test` (builds the bin first);
  a missing build now surfaces as a runtime spawn error, not a link error.
- Churn across 59 files — mechanical, but the behaviour-preservation gate (suites
  green, goldens unchanged) is the proof it is faithful.

## Verification / Closure intent

- The previously-failing suites (`e2e_adr_cli_golden`, `e2e_backlog_filter_alias`,
  and the rest) pass in **both** namespaces without recompiling between runs.
- `e2e_no_baked_manifest_dir` (generalised) passes and would fail if either macro
  creeps back.
- Full e2e suite green, goldens byte-identical — `just gate`.
- `grep -r 'env!("CARGO_BIN_EXE' tests/` returns nothing outside the guard's own
  assembled-fragment needle.

## Follow-Ups
