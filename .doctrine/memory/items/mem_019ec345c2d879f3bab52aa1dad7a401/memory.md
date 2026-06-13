# Nested bwrap userns confines the dispatch worker at the OS floor (D6 LAND)

SL-056 PHASE-12 (O7) spike result. The outer doctrine bubblewrap jail does **not**
seccomp-block `clone(CLONE_NEWUSER)`: `bwrap --unshare-user --ro-bind / / true` run
in-jail exits 0, and `id` inside reports a real `uid=1000` mapping. So a **nested**
`dispatch-worker` bwrap profile is feasible and discharges ADR-006 **D2b**
(worker-on-main / deliberate self-clear) at the OS floor for the **codex/pi
subprocess arm only** (claude's `Agent` is not a subprocess to wrap — its marker-only
altitude is unchanged; D2b stays the deferred residual there).

Canonical profile (the confined spawn template, inline — no flake wrapper needed,
`bubblewrap` is pre-staged in `jailPkgs`):

```sh
bwrap --ro-bind / / --bind "$D" "$D" --bind "$TARGET" "$TARGET" \
      --ro-bind "$MARKER" "$MARKER" --dev /dev --proc /proc \
      --unshare-user --chdir "$D" \
      env DOCTRINE_WORKER=1 $fork_env codex exec "<prompt>"
```

**Load-bearing detail — the marker ro-bind OVERLAYS the rw worktree.** bwrap applies
binds in order, so `--ro-bind "$MARKER"` *after* `--bind "$D"` wins for that single
path: the worktree is rw, but the marker file inside it is read-only. Empirically
(VT-1 conformance, all demonstrated in-jail):

- out-of-tree write → DENIED `Read-only file system` (the `--ro-bind / /` floor); the
  host file is never created (no escape).
- in-tree source write → OK (worktree rw — the worker can edit its fork).
- **`.claude/settings.local.json` write → OK** — it sits in the rw worktree and is NOT
  overlaid, so the harness permission-grant persistence keeps working under
  confinement. **Never ro-bind `settings.local.json`** (§9 constraint) — ro-bind the
  **marker only**, else the interactive permission flow breaks.
- marker `rm` → DENIED `Device or resource busy`; marker overwrite → DENIED
  `Read-only file system`. A confined worker cannot clear its marker to escape
  worker-mode — D6's core invariant holds at the OS floor.

The profile's home is the `/dispatch-subprocess` skill prose (the confined equivalent
of the `env -C "$D"` spawn — design §11 line 152/637), embedded in PHASE-13, NOT a
flake.nix wrapper (design line 639 makes the flake change conditional; the inline
arg-vector suffices). See [[mem.pattern.dispatch.spawn-backend-harness-agnostic-no-free-env-seam]]
(the per-harness floor) and [[mem.pattern.dispatch.claude-subagentstart-worker-identity]]
(the claude arm has no bwrap, marker-only).
