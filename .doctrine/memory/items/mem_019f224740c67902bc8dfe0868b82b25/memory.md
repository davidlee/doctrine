# Claude-arm dispatch: arm-spawn writes the arming base to CWD-detected root, not the coord tree

`doctrine dispatch arm-spawn --base <B> --slice <N>` auto-detects the project
root from CWD (`--path` default). The orchestrator's default cwd during a
`/dispatch` drive is the **session root** (`/workspace/doctrine`), so a bare
`arm-spawn` writes the arming `base` file to the **session-root** arming dir
(`.doctrine/state/dispatch/spawn/base`) — NOT the coordination tree's
(`.dispatch/SL-<N>/.doctrine/state/dispatch/spawn/base`). The `--slice` flag is
**diagnostic-only** (per its own help) and does NOT redirect the write. The coord
tree's `.doctrine/state` is a real dir, not a symlink, so the two arming dirs are
distinct files.

**Why it bites:** the WorktreeCreate (`create-fork`) hook is positional on the
Agent payload cwd — it forks the worker at the `base` oid in the arming dir you
`cd` into. Arm the wrong dir and you either fork off a stale/absent base or (if
that dir has no valid base) get a fallback-to-main worker. This is the same
wrong-base failure family as [[mem.signpost.doctrine.dispatch-claude-arm-wrong-base]],
reached by operator mis-arming rather than lock contention.

**How to apply:**
- Always pass `--path .dispatch/SL-<N>` (the coord tree) to `arm-spawn`, and
  confirm with `cat .dispatch/SL-<N>/.doctrine/state/dispatch/spawn/base`.
- `cd` into the **coord tree's** spawn dir (the path `arm-spawn --path <coord>`
  prints), not the session-root one, before the Agent spawn.
- Verify the fork landed: the base-guard in the worker prompt (`merge-base
  --is-ancestor B HEAD`) + `doctrine worktree verify-worker --base B` fail closed
  if the base is wrong.
- Fix pending: IMP-233 (arm-spawn resolve coord from `--slice`, or refuse a
  wrong-root arming write).

Observed SL-189 dispatch (claude arm): bare `arm-spawn --slice 189` from session
root armed `/workspace/doctrine/.doctrine/state/dispatch/spawn`; corrected with
`--path .dispatch/SL-189`.
