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

## Settled decisions (2026-07-20 preflight)

Option 1 adopted, but the locus and shape are pinned by the code trace:

- **D-A — Locus.** Fix lives in `dispatch_conclude_phase`
  (`src/mcp_server/dispatch.rs`), the claude arm's per-phase conclude. That tool
  flips the sheet with `set_phase_status(&coord.root, …)` → the coord tree; the
  gate reads the completed-set from primary (`registry_completeness(&primary,
  &primary, …)`, `src/dispatch.rs`). The card's "record-boundary" is only the
  claude-arm *manual escape hatch* — not the completion path.
- **D-B — Write BOTH trees, not move.** The coord flip is load-bearing:
  `compute_next_phases` / `dispatch_next_ready` derive completion from the coord
  phase sheet (`phase_projection` → `legacy_status` → `read_phase_status`).
  Moving the flip to primary-only would break the orchestrator's next-phase
  readiness mid-drive. So keep the coord flip and ADD a primary mirror, guarded
  to skip when `primary == coord.root` (solo / no split — no double history
  append). The mirror is safe: `set_phase_status`'s boundary capture self-skips
  while a live coord worktree holds `dispatch/<slice>` (arm guard,
  `src/state.rs`), so it never clobbers the funnel registry rows.
- **D-C — Degrade, don't fail.** If the primary phase sheet is absent, emit a
  named warning (stderr — never the MCP stdout JSON-RPC channel) and let the
  conclude succeed. The boundary row is the durable artifact and prepare-review's
  completeness gate still guards a genuine gap; failing per-phase conclude on a
  missing sheet would halt the drive over an abnormal state (plan materialises
  primary sheets).
- **D-D — Escape hatch untouched; subprocess arm out of scope.**
  `run_record_boundary` stays a pure boundary-correction/bootstrap verb (no
  status flip — it does not assert completion). The codex/pi arm flips status via
  a separate orchestrator step (`slice record-delta` is only its registry write);
  whether that flip reaches primary is verified during implementation — if it
  shares the gap, a follow-up backlog item is filed rather than widening IMP-272.

## Plan sketch

TDD, small:

1. **red** — conclude test in `src/mcp_server/dispatch.rs`: coord/primary split
   fixture, sheets materialised both trees, run `dispatch_conclude_phase`, assert
   the PRIMARY sheet reads `completed` (and coord still does).
2. **green** — in `dispatch_conclude_phase`, after the coord flip resolve
   `primary_worktree(&coord.root)`; if it differs, mirror `set_phase_status`
   there with the same `now`; on error emit a named stderr warning, never fail.
3. **refactor** — comment cites IMP-272; extract a helper only if it earns a name.
4. **docs/memory** — `/dispatch-agent` SKILL step 5 notes the flip now reaches
   primary (retire the manual `-p <primary>` hand-step); update
   `mem.fact.dispatch.prepare-review-reads-primary-phase-status` (the workaround
   is now automatic).
5. **verify** — `just check` then `just gate`; subprocess-arm reach confirmed.
