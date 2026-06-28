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

## D-B3 spike result (2026-06-28) — VIABLE

Confinement is cheap and works. Nested bwrap inside the outer NixOS jail is
permitted (unprivileged userns nesting; bwrap 0.11.2). Mechanism, ~12 lines
wrapping only the worker's `pi` exec (the orchestrator-classed fork stays
unconfined in `$ROOT`):

```
bwrap --ro-bind / / --dev /dev --proc /proc --tmpfs /tmp \
      --bind "$D" "$D" --chdir "$D" --die-with-parent \
      --setenv DOCTRINE_WORKER 1 pi …
```

`--ro-bind / /` makes the whole fs read-only; `--bind "$D" "$D"` re-grants rw to
only the worker tree (its in-tree `target/` rides along — covers D-B1's build dir
for free). `OUT`/`PI_FIFO` are host-`/tmp` fds opened by the spawning shell
before bwrap execs, so the inner `--tmpfs /tmp` does not sever them.

Verified against worktree stand-ins: worker can write `$D` + `$D/.pi-session`;
writes to `.doctrine/`, the main tree, and sibling worktrees all fail
`Read-only file system`; `git` + `doctrine` still run from the ro `/nix/store`.

Landed as `scripts/pi-spawn-confined.sh` — a copy of `pi-spawn.sh` (the live
script left untouched while in active dispatch use). Promote into `pi-spawn.sh`
once the live run frees it.

Residual unknown, deferred to live dispatch (per "not exercised until /dispatch
runs on doctrine"): whether `pi` needs a writable `$HOME` dot-dir beyond
`--session-dir` (mitigated with `--no-extensions --no-skills --no-themes
--offline`). Contingency: bind a `--tmpfs` at the specific dot-path if so — do
NOT tmpfs `$HOME` wholesale (would mask the ro `~/.cargo/bin/doctrine`).
Discharges ADR-006 D2b at the OS level.
