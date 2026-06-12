# Removing a dispatch worktree leaves env!(CARGO_MANIFEST_DIR)-baked test binaries pointing at the deleted fork path — false RED until recompiled

`crates/cordage/tests/denylist.rs` resolves its walk root with
`PathBuf::from(env!("CARGO_MANIFEST_DIR"))`, evaluated at **compile** time. A
dispatch worker builds the workspace inside its worktree fork
(`.worktrees/<branch>/...`) against the **shared** jail target
(`~/.cargo/doctrine-target-jail/debug`), so the cached test binary bakes the
fork's manifest path. After the orchestrator removes the worktree, re-running
`just check` from the parent tree reuses that cached binary, walks the now-deleted
fork path, finds 0 files, and fails the test's own root-resolution sanity guard:

```
denylist walk found no src/lib.rs — root resolution is wrong
  (got 0 files under /workspace/doctrine/.worktrees/<branch>/crates/cordage)
```

This is a **false RED**, not a real failure — the inverse of the false-GREEN in
[[mem.pattern.dispatch.shared-target-false-green-touch-rerun]]. Fix: force a
recompile so `CARGO_MANIFEST_DIR` re-bakes to the real path — `touch
crates/cordage/{src/lib.rs,tests/denylist.rs}` then re-run `just check` (or
`cargo clean -p cordage`). Orchestrator hygiene: after removing worker worktrees,
expect any `env!`-path-baked test to need a rebuild before the close-time
`just check` is trustworthy. Surfaced at SL-053 close. cordage is gated by
`just check` since SL-052 ([[mem.pattern.build.just-check-workspace-gates-members]]).
