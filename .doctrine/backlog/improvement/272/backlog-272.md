# IMP-272: dispatch funnel: per-phase completed flip must reach primary tree — prepare-review completeness gate reads phase status there, not the coord tree

## Symptom

At SL-205 conclude, `dispatch sync --prepare-review` failed the completeness
gate for **all four** phases: `conformance registry incomplete: recorded row for
PHASE-0N, which is not a completed phase`. Each phase had been flipped
`completed` via `doctrine slice phase --status completed SL-205 PHASE-0N` during
the drive.

## Root cause

`slice phase --status` writes **runtime tracking**
(`.doctrine/state/slice/<n>/phases/*.toml`), which is **per-worktree**. On the
claude arm the orchestrator drives from the coordination worktree, so the flips
landed in the **coord tree's** runtime state. But `record-boundary` double-writes
the conformance registry into the **primary tree**, and the prepare-review
completeness gate reads phase-completion status from the **primary tree** too.
Result: registry rows present in primary (from record-boundary), phase status
still `planned` in primary (flips went to coord) → gate sees rows for
"not completed" phases and bails.

## Workaround used (SL-205)

Re-flipped all four with an explicit primary root:
`doctrine slice phase --status completed SL-205 PHASE-0N -p /workspace/doctrine`,
then re-ran prepare-review (passed, 5 refs). Cost ~4 probe turns to locate the
tree split.

## Fix options

1. **record-boundary co-writes phase status** into the primary tree (it already
   reaches the primary registry there) — the flip and the registry row land
   together, atomically, in the tree the gate reads.
2. **Funnel documents the split** — the `/dispatch-agent` funnel's per-phase
   `slice phase --status completed` must target the primary root (`-p <primary>`),
   not the coord tree.
3. **Gate reads coord phase status** — least attractive; the registry already
   lives in primary, so completion status should too.

Recommendation: option 1 — remove the hand-step entirely; the completed flip is
implied by a recorded boundary. Sequence independent of SL-205.

Origin: SL-205 dispatch conclude (RV-256 audit harvest). See also RFC-011
case-notes `[dispatch; SL-205-conclude-phase-status-split]`.
