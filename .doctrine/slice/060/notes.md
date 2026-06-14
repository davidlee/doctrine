# SL-060 — durable implementation notes

## PHASE-01 (canon-first spec amendment) — DONE, VH-1 signed 2026-06-14

Canon outcomes are authored canon now (durable): PRD-011 §1/§2/§3 + REQ-258
(FR-009) + REQ-097 cross-ref. Commits fa38369 (amendment) + 3bf5b99 (boundary
tightening). Read them via `doctrine spec show PRD-011` / `requirement-258.{toml,md}`.

Durable carry-forwards (the gitignored phase sheet is not a safe home):

- **Endpoint rule is a CLOSED ALLOWLIST.** Valid `needs`/`after` endpoints = slice +
  the backlog kinds; **every other admitted kind refused** (governance, spec,
  requirement, knowledge, drift, review, reconciliation). Stated positively on
  purpose — a denylist-by-example rots as the corpus admits more kinds. REQ-258 #3
  enumerates the full refused set.

- **PHASE-03 test-coverage delta (not a logic change).** The D2 predicate is already
  allowlist-by-construction (design §5.2 lines 198–199, D2 line 273), so drift/review/
  rec are refused for free. But plan VT-2 names only 4 refusal examples (gov/spec/req/
  knowledge); REQ-258 #3 now widens the *verification* expectation. PHASE-03 refusal
  tests must cover a representative beyond the 4 — pick a resolvable non-work entity
  not in the original list (RV/REC if they're in `integrity::KINDS`; else that hits the
  unresolvable-target path, also refused). Free-text and self-edge refusals unchanged.

- **Why non-work is refused (don't re-litigate).** A non-terminal non-work predecessor
  genuinely blocks (`channels::blocked_by` filters by StatusClass≠Terminal, not by
  kind) — "allowed-but-inert" was refuted. Cross-tier governance→work gating is a
  *distinct* future surface (IMP-047 labelled `gates` + non-actionable status-class),
  deliberately not this `needs`/`after` surface. slice→requirement lineage uses
  `descends_from`, never `needs`. Full rationale: REQ-258.md Rationale.

- **Filed this session:** IMP-058 (render the requirement `.md` prose tier — currently
  unreachable via `spec show`), relates to IMP-057 (requirement authoring skill).

## PHASE-02/03 dispatch carry-forwards (for audit)

- **PHASE-02 landed b0d3e3d** — `src/dep_seq.rs` leaf (DepSeq/AfterEdge/RelEdit/read/
  strict append, non-destructive refuse message). Backlog repointed to leaf type+write;
  kept single `read_item`+`dep_seq_for` for `promoted` (one parse, F3). INV-2 backlog/
  priority/order goldens byte-identical.
- **PHASE-03 landed ec0d14b** — generic `doctrine needs`/`after` ride `link`'s
  cross-kind resolver; work-like gate = `is_work_like(kind)` (slice + SL/ISS/IMP/CHR/
  RSK/IDE) in main.rs, the single widen-later guard. Backlog verbs unchanged at source
  (already delegated to leaf at base; cycle refuse intact). Slice scaffold seeds
  `[relationships]` both arrays before `[[relation]]`; slice show/--json surface dep/seq.
- **AUDIT FLAG — existing golden amended by design.** `tests/e2e_relation_migration_
  storage.rs::assert_slice_shape` asserted (SL-048 "the cut") the slice template emits
  NO `[relationships]` header. SL-060 §5.3/E9 deliberately reinstates the table for the
  dep/seq PAYLOAD axis (structural axes stay cut). Worker rewrote the golden to assert
  the table IS present (needs/after) while structural axes (specs/requirements/
  supersedes/governed_by/related) remain cut + link-guidance comment retained. NOT an
  INV-2 behaviour-preservation golden (those stayed green); design-mandated, transparent.
- **VT-2 widened coverage met** — minted a resolvable RV-001 in the test root; refused
  by the work-like kind assertion (not the unresolvable path) + a unit test sweeps
  `integrity::KINDS` asserting only slice+5-backlog pass. Satisfies REQ-258 #3 closed
  allowlist.
