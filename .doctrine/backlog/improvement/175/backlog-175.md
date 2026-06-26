# IMP-175: Solo phase-binding stamps stale code_start_oid when edge advances before land

Surfaced by the SL-138 audit (RV-165 F-1/F-2).

## Problem

`capture_phase_boundary` (`src/state.rs:414`, SL-147 PHASE-04) stamps
`code_start_oid` = HEAD at the **InProgress** flip and `code_end_oid` = HEAD at
**Completed**. The recorded range is therefore `[HEAD-when-phase-started,
HEAD-when-phase-landed]`. When a slice is developed across a busy period — other
slices land on `edge` between a phase's in_progress stamp and its (rebased) land
— those foreign commits fall inside `start..end`, and `slice conformance`
attributes them to the phase.

## Evidence (SL-138)

Each SL-138 phase is exactly one non-merge feat commit, but the recorded
boundaries swept in foreign history:
- PHASE-01 `53e04df9..f4104f55` → +3 SL-154 (close/reconcile/audit) commits.
  True delta: `5cb84f3a..f4104f55`.
- PHASE-02 `f4104f55..42de85c4` → +27 SL-156/SL-154/IMP-174/RFC-005 commits.
  True delta: `83b9cea2..42de85c4`.
- PHASE-03 `6703ddc5..1ed4c750` → clean (manually re-recorded during execute,
  per the handoff — the only one that was corrected).

Conformance reported **49 undeclared** (vs 4 after correction) — enough noise to
bury genuine scope creep. The audit corrected P1/P2 via `slice record-delta`.

## Candidate fixes

- Re-derive `code_start_oid` at **Completed** time as the landed feat commit's
  first parent (the actual delta base), rather than trusting the in_progress
  stamp — robust to rebased landings.
- Or detect divergence at completion (stamped start not an ancestor-by-one of
  end) and warn loudly so the operator runs `record-delta`.
- Relates to ISS-052 / IMP-171 (registry write fidelity on the dispatch arms).

The manual `record-delta` escape hatch already exists and is the current
mitigation; conformance surfacing the pollution as `undeclared` is the safety
net. This item is about making the **automatic** solo capture correct so an
auditor is not the last line of defence.
