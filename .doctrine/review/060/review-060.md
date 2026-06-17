# RV-060 Audit Brief — SL-087 reconciliation

**Subject:** SL-087 — Boot snapshot token efficiency & correctness hardening
**Scope:** Single-phase implementation (PHASE-01): trim Memory section from full metadata table to reference line + compact key list
**Evidence base:** git commit `223dc36b`, `just check` output (1613/1614 pass, 1 pre-existing unrelated failure), regenerated `boot.md`
**Design review:** RV-058 (5 findings, all resolved via design remediation — verified terminal)

## Lines of attack

1. **Design conformance (F-1 through F-5 from RV-058):** Verify every design-review finding's remediation is faithfully implemented — uid fallback, section_or_marker routing, key-ascending sort, internal filtering, pull-pointer reference line.

2. **Phase criteria (VT-1 through VT-4, VA-1 through VA-2):** Verify all verification criteria are met or properly dispositioned.

3. **Code quality:** No dead_code, no parallel implementation, clean separation between boot_keys (memory) and produce (boot), consistent with existing seams.

4. **Behaviour preservation:** Confirm existing tests unchanged — the `produce_markers_a_non_exec_source_and_carries_the_exec_path` test validates VT-2 (empty corpus → marker).

5. **Output verification:** Regenerated `boot.md` Memory section is ~22 lines, key-ascending, reference line present.

## Synthesis

SL-087 is a clean, single-phase implementation with zero code defects. The
implementation matches the settled design (post-RV-058) exactly. Two small
functions (`boot_keys` on memory, updated `Memories` arm in produce) carry the
entire change — narrow, testable, and reuse existing seams (`collect_all`,
`section_or_marker`).

All five RV-058 design-review findings (F-1 through F-5) are faithfully
implemented: uid fallback for keyless memories, section_or_marker routing for
error handling, key-ascending sort, full `pub(crate)` contract with internal
filtering, and the pull-pointer reference line. The regenerated `boot.md`
Memory section is 22 lines (1 reference + 21 keys) — a ~58% reduction from
the previous ~50-line full metadata table.

All phase verification criteria are satisfied:
- VT-1 through VT-4: all tests pass (unit + integration + regression)
- VA-1: boot.md Memory section renders 22 lines with reference + key list
- VA-2: just check is green; just gate blocked by a pre-existing unrelated
  `e2e_memory_sync` failure (confirmed on clean main, not SL-087 implicated)

The single standing risk is the pre-existing `sync_produces_all_shipped_dirs`
gate failure — tolerated per F-2, not within SL-087 scope to fix. No other
findings or concerns.

## Reconciliation Brief

### Per-slice (direct edit)
*No per-slice design changes needed — the design.md is current and accurate.*

### Governance/spec (REV)
*No governance or spec changes needed.*

### Notes for /reconcile
- RV-060 is clean (0 unresolved blockers, 2 terminal findings: 1 aligned, 1 tolerated)
- The pre-existing `e2e_memory_sync::sync_produces_all_shipped_dirs` failure exists on clean main — SL-087 is not the cause
- No design or governance artifacts need amendment
- Slice can proceed directly to /close after reconciliation sign-off

## Reconciliation Outcome

No per-slice design edits needed — the design.md is current and accurate.
No governance/spec (REV) items exist — all findings were withdrawn or tolerated
with rationale:

- **F-1 (aligned):** All RV-058 design findings faithfully implemented. No writes.
- **F-2 (tolerated):** Pre-existing `sync_produces_all_shipped_dirs` failure —
  not within SL-087 scope to fix; rationale in finding disposition.

Reconcile pass complete — handoff to /close.
