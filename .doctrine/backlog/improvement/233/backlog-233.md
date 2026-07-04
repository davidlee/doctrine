# IMP-233: arm-spawn should target coord tree from --slice or refuse a wrong-root arming write

## Problem

`doctrine dispatch arm-spawn --slice N` auto-detects the project root from CWD.
When run from the **session** root (the orchestrator's default cwd, which is the
primary tree, not the coord tree), the `--slice` flag is diagnostic-only and
does NOT redirect the write — so the arming `base` file is written to the
**session-root** arming dir, not the coord tree's. The worker then reads no
arming base and the spawn fails.

Witnessed in the SL-189 orchestrator claude-arm drive:
- Wrong arm written to session-root `.dispatch/` instead of coord-tree
  `.dispatch/SL-189/`
- Cost: inode check + re-arm with explicit `--path .dispatch/SL-N` + stray-base
  cleanup (~1 extra Bash round-trip + reasoning)

The `--slice` flag's help text already says it's diagnostic-only, so this is
expected behaviour — but the ergonomics are misleading: a user who passes
`--slice N` reasonably expects the arming write to target slice N's coord tree.

## Fix direction

- **`--slice` resolves the coord tree**: `arm-spawn --slice N` looks up the
  live coord tree for slice N (`dispatch/ N` branch), resolves its worktree
  path, and writes the arming base there. This is the intuitive behaviour.
- **Or refuse**: if CWD root != the coord tree resolved from `--slice`, print a
  clear error: "arming write targets the session root; use `--path <coord>` to
  target the coordination tree."
- Prefer the resolve-and-redirect path: it matches user expectation and removes
  the footgun.

## Related

- RFC-011 case-notes: `[dispatch; sl189-orch-claude-arm]`
- IMP-257 (mid-drive authored-commit guard — adjacent base-correctness concern)
