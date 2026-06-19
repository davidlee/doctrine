# CHR-014: Tests resolve install/templates via env!(CARGO_MANIFEST_DIR) — shared CARGO_TARGET_DIR bakes dead worktree path

## Symptom

`cargo test` panics:

```
read template /tmp/<removed-worktree>/install/templates/slice.toml:
  No such file or directory (os error 2)
```

from `template_text` / `templates_dir` in `tests/e2e_relation_migration_storage.rs`
(also any test reading `install/templates/` or `.doctrine/` this way).

## Root cause

`templates_dir()` resolves the path at **compile time**:

```rust
PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("install/templates")  // :45
PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".doctrine")          // :35
```

The jail sets a **shared** `CARGO_TARGET_DIR` (`/home/david/.cargo/doctrine-target-jail`)
across every worktree. When `cargo test` is run from worktree W, the test binary
bakes `CARGO_MANIFEST_DIR = <W>` and lands in the shared target. cargo's fingerprint
treats it as fresh for identical source, so a later `cargo test` from a *different*
tree (or the main tree) **reuses the stale binary** that points at W. Once W is
removed (`git worktree remove`), the path is dead → panic.

Hit during SL-111 close: a baseline `cargo test` in a throwaway `/tmp` worktree
contaminated the shared target; the main tree's `just check` then failed on a path
under the removed worktree. Forced rebuild (`touch` the test + rebuild) fixed it,
but the fragility is latent and re-arms on any cross-worktree test run.

## Fix options

- Resolve `install/templates` / `.doctrine` from a **runtime** root (CWD walk to the
  repo root, or an explicit `DOCTRINE_ROOT` env) instead of `env!(CARGO_MANIFEST_DIR)`.
- Or give the worktree-spawning flows (dispatch / baseline checks) their **own**
  `CARGO_TARGET_DIR` so binaries never cross trees.

Related but distinct: ISS-024 (stray `.doctrine/` dirs in worktrees break the
corpus scanner), ISS-028 (worker-marker confinement breaks CLI-shelling tests).
This one is the compile-time-path-vs-shared-target axis.
