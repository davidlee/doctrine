# SL-101 notes — Inquisition (RV-082)

2026-06-18 — design inquisition complete. 4 findings raised, all resolved.

## Resolved

- **F-1 (BLOCKER → verified):** Default unit `espresso_shots`. User ruled: design is
  forward-looking authority; SPEC-020 will be amended during reconciliation. No code
  change needed.
- **F-2 (BLOCKER → tolerated/verified):** Value facet (`magic_beans`) has no governing
  spec yet. User ruled: Value facet stays; governing PRD+SPEC to be authored and
  SPEC-020 amended during reconciliation.
- **F-3 (minor → fix-now/verified):** Verification alignment table (§8) has
  FR-004/NF-003 mapped to wrong test indices. Fix: FR-004 → E4,E17; NF-003 → E16,V7;
  §11 "tested in E15" → "tested in E16".
- **F-4 (minor → fix-now/verified):** E17 claims unknown keys survive round-trip but
  `EstimateFacet` has no `_extra` field. Fix: correct E17 to test parse-tolerance
  only. §3.4 prose: "survive round-trip" → "tolerated at parse; v1 does not persist."

## Standing

- SPEC-020 amendment needed during reconciliation: change default unit from
  `high_caffeine_hours` to `espresso_shots`; add Value facet coverage.
- Value facet needs a governing PRD+SPEC authored (reconciliation deliverable).
- F-3 and F-4 design.md edits are mechanical — do them before implementation.

## PHASE-01 complete — 2026-06-18

- `src/estimate.rs`: 27 tests green, clippy clean
- `mod estimate;` added to `src/main.rs` (sequencing fix — was erroneously in PHASE-04)
- `#![allow(dead_code)]` on estimate.rs (temporary; lifted in PHASE-03 when dtoml.rs wires config imports)
- `plan.toml` updated: mod declarations moved from PHASE-04 to PHASE-01/02 exit criteria
- Handover updated for PHASE-02 entry
