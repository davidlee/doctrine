# IMP-004: Jail dispatch isolation spike: per-worktree target and bwrap confinement

Implements ADR-008. Project-local to doctrine-the-repo: makes parallel worktree
dispatch build-safe and confined inside the bubblewrap jail.

Scope:
- **Per-worktree `CARGO_TARGET_DIR` (D-B1):** nested under the jail-redirect root,
  keyed by worktree/branch, set at worker spawn. Restores parallel builds, kills
  cross-branch thrash, gives correct per-worktree `CARGO_BIN_EXE`.
- **No in-jail `cargo install` (D-B2):** workers/tests run the per-worktree
  build-target binary; ro `~/.cargo/bin/doctrine` stays the stable entrypoint.
- **Per-worker bwrap confinement — SPIKE FIRST (D-B3):** nested bwrap rw-mounting
  only the worktree + its target, ro everything else (incl. main). Discharges
  ADR-006 D2b at the OS level. **Timeboxed spike**; if too costly, back out to
  D-B1 + the D2a CLI guard (IMP-002) and leave D2b deferred.
- **`sccache` (D-B4):** deferred until cold builds cause pain; noted.

Not exercised until `/dispatch` runs on doctrine itself (IMP-003). Governing:
ADR-008 (discharges ADR-006 D2b).

## Motivating evidence — correctness hazard, not just build thrash

The shared `CARGO_TARGET_DIR` (one cache across every worktree) is not only a
performance/thrash problem; it is a **correctness** problem. Cargo's fingerprint
reuses a *test* artifact compiled in tree W when tests run from another tree, so a
test binary can hold code/fixtures that no longer match HEAD → false RED **and**
false GREEN. D-B1 (per-worktree target) is the structural cure: no binary crosses
trees, so no stale reuse.

Two axes of this footgun, observed directly:

- **#1 path-baking (FIXED, CHR-014, distinct):** `env!("CARGO_MANIFEST_DIR")` baked
  the building tree's path; a reused binary read a dead worktree path once that tree
  was reaped. Resolved by runtime path resolution — *not* in IMP-004's scope, and it
  does **not** touch axis #2.
- **#2 stale test artifact (IMP-004's domain):** the compiled test binary itself is
  stale. Confirmed this session — `just gate` reds on
  `e2e_dispatch_sync::record_boundary_also_writes_the_arm_neutral_registry`, a test
  name **absent from current source** (a deleted test still executing from a reused
  artifact). Earlier instance: a `run_record_boundary` change read as a "P04 defect"
  until a forced rebuild. Mitigation today is fragile discipline only:
  `just rebuild-stale` retouches the **bin**, not test binaries — clearing a stale
  test artifact needs `cargo clean -p doctrine` or touching the specific test source.
  Compounded by tests shelling the RO `~/.cargo/bin/doctrine` (D-B2 keeps the
  build-target bin as the run target).

See memories `mem.fact.build.rebuild-stale-skips-test-binaries` (axis #2) and
`mem.fact.testing.runtime-manifest-dir` (axis #1, the CHR-014 fix).
