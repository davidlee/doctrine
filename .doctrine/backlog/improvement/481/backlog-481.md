# IMP-481: Assess and author governance coverage for the RV kind

`ADR-007` is the only artifact that owns the RV kind; no PRD or SPEC covers it
(`grep` over spec titles finds none). The ADR has drifted from the implementation
in at least two places (D-C8 empty ledger, D-C10 warm-cache) and its shipped
protocol doc drifted too.

The RFC-032 programme intends each slice to leave behind spec coverage. First step:
a spec-coverage assessment (see `IMP-295`'s skill) for the review surface, to decide
whether a PRD/SPEC owns the kind or ADR-007 stays sole authority and is kept
current. See RFC-032 `research.md` F13.

## Outcome (2026-09-26) — assessment done, user agreed

Bounded governance-coverage census (skill `/spec-coverage-assessment`), confirming
RFC-032 `decision-frontier.md` D13 and fixing its placement.

**Census.** ADR-007 remains sole owner. No tech spec anchors any review code
(`src/review.rs`: ~3 200 code + ~2 550 test lines, one module). SPEC-029 claims
only the design-run minting side; PRD-013 and PRD-019 mention review in prose
only. Name collisions, not RV: `src/ledger.rs` (dispatch run-ledger),
`src/finding.rs` (doctor findings), `review/<N>` refs (SPEC-021/022).

**Ranked gaps.** (1) ledger mechanism — schema, transitions, `done`, baton/lock/CAS,
cache/`prime`, read surface, close gate: dark; (2) `install/review-ledger.md`
drift, unchecked; (3) the RV side of the design-run bind (RFC-032 D6): no home.

**Boundary (agreed).**
- One new tech spec, "Review ledger": **container**, `parent = SPEC-003`
  (precedent SPEC-028, SPEC-029). Rejected: component under SPEC-004 — RV carries
  its own concurrency model, cache and close gate, not just a kind surface.
- One spec, not several: D4's pure-core / shell split is a module seam, not a spec
  boundary. Reservation stays in SPEC-008; the design-run bind stays in SPEC-029
  (RV side stated in the new spec; both revised in slice 3).
- No new PRD (precedent SPEC-016 carries no `descends_from`). Open for
  `/spec-tech`: leave descent empty (recommended) vs descend from PRD-013.
- ADR-007 slimmed by REV to decisions; mechanism clauses (D-C5, D-C8, D-C10,
  D-C1/C7) cite the spec.
- Authored in RFC-032 slice 1 (Ledger v2) so it describes v2, not v1; extended per
  slice. Anchors: review modules, review MCP tools, the close-gate predicate in
  `src/slice.rs` — verify liveness at authoring.
- Drafting sources: ADR-007, RFC-032 `research.md` §2 + F1–F13,
  `decision-frontier.md` D1/D2/D4/D8/D10/D15, DEC-125, DEC-138, IMP-001, SL-061,
  SL-147, RFC-004.

**Out of scope, surfaced.** No spec governs `doctor`, so D13's doc-drift check has
no spec home — captured separately.
