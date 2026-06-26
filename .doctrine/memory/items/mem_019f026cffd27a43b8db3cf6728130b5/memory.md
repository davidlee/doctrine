# In-tree per-worktree target: isolation by construction, no shared CARGO_TARGET_DIR

Each worktree builds into its **own gitignored in-tree `target/`** (cargo's
default). There is **no shared `CARGO_TARGET_DIR` redirect** — SL-156 retired the
`flake.nix` export (PHASE-01) and removed the platform coupling (PHASE-03);
ADR-008 D-B1 is the governing decision (D-B5 keeps `target_dir_for_branch` as a
framework primitive).

**Consequence — the whole shared-target stale-artifact hazard class is gone by
construction:**
- No two worktrees share a target dir, so cargo's fingerprint can never serve one
  tree's binary/test-artifact to another (no false-RED / false-GREEN from sharing).
- Jail (`/workspace/doctrine/target`) and host (`/home/.../doctrine/target`) are
  **distinct physical dirs** → no cross-mount `CARGO_BIN_EXE` clobber (the original
  flake concern is satisfied *by* in-tree targets, not by a redirect).
- A fork's `target/` lives inside the fork → reaped with `worktree remove`. No
  orphaned-target GC, no `env!`-baked-path-points-at-deleted-fork false-RED.
- `./target/debug/doctrine` is the live binary again after `cargo build`. The
  `just rebuild-stale` / touch-`main.rs` rituals are retired.

**Caveat (R5 — env lag).** The `set-env` removal applies at **jail launch**, so a
session started before the relaunch still inherits the old
`CARGO_TARGET_DIR=…/doctrine-target-jail`. The in-tree reality is live after the
next jail relaunch (first build per tree is cold; the old
`~/.cargo/doctrine-target-jail` is abandoned — remove it out-of-band).

**Still true under in-tree (NOT superseded — these never depended on sharing):**
- [[mem.fact.testing.runtime-manifest-dir]] — resolve test paths via the runtime
  `CARGO_MANIFEST_DIR` / `test_support::repo_root()`, never the `env!` macro; the
  `e2e_no_baked_manifest_dir` guard still bans it. A single tree's incremental
  build can still serve a stale test binary after a source/fixture edit.
- [[mem.pattern.jail.stale-test-fixture-vocabulary-change]] — a not-rebuilt test
  binary embeds the old fixture corpus; and `just gate 2>&1 | tail` masks a
  non-zero exit (check `$?` directly) — both independent of the shared target.

Supersedes the shared-`CARGO_TARGET_DIR` hazard cluster (SL-156 PHASE-04 triage).
