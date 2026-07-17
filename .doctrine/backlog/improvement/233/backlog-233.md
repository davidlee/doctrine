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

## Related sighting — read side, silent (SL-221 audit, 2026-07-18)

The same CWD-root auto-detect footgun bites **read** verbs, and worse: silently.
During the SL-221 audit, `dispatch candidate status`/`admit` run with the shell
inside a linked worktree (the detached candidate worktree
`.doctrine/state/dispatch/candidate/cand-221-review-001`, a main-based snapshot
predating the `create` write) resolved `root` to that worktree, whose
`.doctrine/dispatch/221/candidates.toml` is absent. `read_candidates` returns
empty-when-absent by design (VT-2), so status printed "(none recorded)" instead of
erroring — read as a phantom "read/write ledger split" and cost ~15min of
misdiagnosis before recognising it as this CWD-root footgun. Run from the primary
root the row is present and `admit` works.

Distinct from the arm-spawn write case in two ways worth folding into the fix:
- **Silent, not a spawn failure.** The write case fails loudly downstream (worker
  reads no base); the read case returns a valid-looking empty result. Any refuse/
  redirect fix should also cover read verbs that resolve runtime state
  (`.doctrine/dispatch/**`, phase sheets) — an empty-but-present-elsewhere result
  is the trap.
- **Generalises past arm-spawn.** The root cause is `root = auto-detect(CWD)` for
  any verb touching per-slice runtime state, not arm-spawn specifically. Consider a
  cross-cutting guard: when CWD resolves to a linked/detached worktree and the verb
  reads or writes coordination-tier state, warn (or resolve from `--slice`) rather
  than silently reading the wrong tree.

Same session also mis-read `slice status 221` as `ready` from the candidate tree —
same footgun, same class.

## Related

- RFC-011 case-notes: `[dispatch; sl189-orch-claude-arm]`,
  `[audit; SL-221-audit-rv283 · CORRECTION at reconcile]`
- IMP-257 (mid-drive authored-commit guard — adjacent base-correctness concern)
