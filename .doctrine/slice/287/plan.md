# Implementation Plan SL-287: Commitment ledger for plan authoring and review

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Reference forms: `lib:reference/glossary.md` § reference forms. -->

## Overview

Two phases: first the normative home (`lib:reference/commitment-ledger.md`) and
its publication, then the skills that route to it. Guidance only; the one code
change is a ship test that keeps the doc's TOML example parseable.

This plan dogfoods the procedure it ships. `commitment-ledger.toml` was
extracted from the locked design before phases were authored, satisfied while
writing `plan.toml`, and cross-checked before the first commit (coverage
scripted; content and proportionality by hand). Its plan review is the two-RV
shape of design sec-4. The trial evidence `RFC-029` § 9 wants starts here.

## Sequencing & Rationale

```text
PHASE-01  doc + manifest entry + ship test ──► lib:reference/commitment-ledger.md resolves
              │  (C-22, C-23)
              ▼  PHASE-02/EN-1 gates C-29, C-31
PHASE-02  five skill pointers · /plan step-4 fix · glossary · using-doctrine
```

- **Why this order.** A skill citing `lib:reference/commitment-ledger.md`
  before its manifest entry exists fails `every_shipped_lib_citation_resolves`,
  so the pointers (C-31) and any glossary citation (C-29) need the published
  address (C-22, C-23). That is the only correctness edge; the ledger records it
  as `needs`, and PHASE-02/EN-1 is its gate.
- **Parallel candidate, not taken.** The `/plan` step-4 correction (C-30) needs
  nothing and touches a disjoint line; it could run beside PHASE-01. Two
  short phases on one file family make serial cheaper.
- **Why not one phase.** One phase would end green too. Two keep the doc's
  content review (PHASE-01/VA-1, the heaviest check) separate from the pointer
  wiring, and give the procedure a real cross-phase edge to exercise.
- **Each phase ends green.** PHASE-01 adds a doc nothing cites yet (no orphan
  check exists); PHASE-02 adds citations of a published address.

## Notes

- Cross-check before the first commit caught a cited-but-wrong criterion
  (PHASE-01/EX-6 counted four staleness conditions; the design has five), two
  lead-in-only quotes (C-15, C-20, widened to the full passage), and an
  unattributable VT (PHASE-02/VT-1 pointed at a file the slice never edits).
- VT keyword floors here are weak by nature (prose deliverables); the doc's
  substance is proved by PHASE-01/VA-1 against the ledger's quotes.
