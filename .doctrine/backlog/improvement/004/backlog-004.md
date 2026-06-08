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
