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

## Dispatch coordination — 2026-06-18

- Coordination worktree created at `/tmp/doctrine-dispatch-101` (base `d22ab0be`)
- Dispatch ref: `refs/heads/dispatch/101`
- Phase sheets materialised in coordination worktree
- Handover rewritten with dispatch mandate, pre-distilled PHASE-02 worker prompt, and full funnel instructions
- Note: `1ffdeb3f` (PHASE-01) was followed by `49f41684` (SL-099 fix) and `d22ab0be` (SL-099 candidate merge) on main — coordination base is `d22ab0be`

## Audit complete — 2026-06-18 (RV-085)

Conformance audit on `candidate/101/review-001` (tip after audit: `a50f7092`).
4 findings raised, all terminal (verified):

- **F-1 (major → fix-now):** `dtoml::parse()` eagerly ran `resolve_confidence()?`,
  coupling the shared config reader (conduct/verification/coverage_store) to
  estimation-config validity — violated design §3.3 "no runtime effect in this
  slice". Fixed: removed eager call; marked v1-unused facet API with the existing
  `#[cfg_attr(not(test), expect(dead_code))]` convention; added regression test.
  Commits `8598dbca` + `a50f7092` on `candidate/101/review-001`.
- **F-2 (minor → fix-now):** dead `resolve_unit` discards in `parse()` — same edit.
- **F-3 (minor → verified):** design §3.3 `resolve_confidence` signature shows
  bare `(f64,f64)` but docstring says "validated"; impl correctly returns
  `Result`. → reconcile: edit design §3.3.
- **F-4 (minor → verified):** design §6.1 `SliceDoc` derive still lists `Eq`,
  impossible with f64 facets; impl correctly dropped `Eq`. → reconcile: edit
  design §6.1.

Evidence: 1768 bin tests pass, clippy zero, fmt clean on the candidate.
Memory recorded: `mem.pattern.doctrine.dtoml-shared-reader`.

**Handoff to /reconcile:** design.md §3.3 + §6.1 edits (per Reconciliation Brief
in review-085.md). SPEC-020 amendments from RV-082's standing list remain the
reconcile-stage spec work. No `blocker` outstanding.
