# Comparison ledger capture

## Context

RFC-019 Phase A (externally reviewed, RV-260 all-verified). The comparison
ledger is the capture side of comparison-based value elicitation: append-only
session files of pairwise judgements, authored evidence for the Phase B
constraint layer. This slice is **pure capture** — no scoring change, no
inference, no queue. ADR-015 amendment rides Phase B (staged as REV-022);
nothing here consumes the ledger.

## Scope & Objectives

- **Session-file schema** (RFC-019 T2): `.doctrine/comparisons/<date>-<uid>.toml`,
  `[session]` header (rater/audience, date, session uid) + append-only
  `[[judgement]]` rows. Schema-versioned like other doctrine TOML.
- **Full typed row from day one** (lossless capture):
  `{uid, a, b, preferred, domain, frame (closed vocab per domain),
  form: order|ratio, lens?, rater (mandatory; human|agent), note, date}`.
  Phase A *elicits* only value-domain order rows; the schema carries the rest.
- **Capture verb**: `doctrine value compare <A> <B> --prefer <A|B>
  [--lens L] [--by RATER] [--note …]` — verb naming is a design-time question
  (RFC-019 flags `value compare` as possibly wrong-shaped once the estimate
  domain arrives; a domain-neutral `doctrine compare` may age better).
- **List verb**: `doctrine value comparisons [<ID>]` — evidence listing,
  per-item filter, active/superseded visibility.
- **Domain admissibility at capture** (A2/A4): value-domain rows admit only
  commensurable value-bearing pairs; records refused with reason (their worth
  is derived, A2). RSK admissibility per REV-022 Q2 adjudication.
- **Row identity**: per-row `uid` via the date/uid pattern — supersession
  ordering and tombstone references (Phase B) resolve against it; minted here.
- **Session mechanics**: ad-hoc single comparisons land in an implicit
  session (per-invocation session-of-one or per-day default file — mechanics
  settled at design, RFC-019 T2).

## Non-Goals

- Constraint propagation, bounds, projection, contradiction surfacing —
  Phase B.
- Supersession/tombstone *resolution* semantics — Phase B (whether a
  tombstone-append verb ships here is a design question; resolution does not).
- Elicitation queue, pair selection, binary insertion — Phase C.
- Estimate/risk domains — capture schema admits the typing; no elicitation.
- Any change to scoring, `survey`/`next`/`explain`, or the priority graph.
- Web/session surfaces (Phase D+).

## Affected surface

- New leaf/engine module for ledger schema + read/write (`src/` — exact home
  at design; ADR-001: below command tier, pure core over parsed rows, disk at
  the scan seam).
- Command tier: new verb(s) under `value` (or a new noun, per naming
  question).
- `.doctrine/comparisons/` — new authored directory (committed, diffable).
- Kind admissibility touches `src/kinds.rs` constants read-only
  (VALUE_BEARING).

## Risks, assumptions, open questions

- **OQ-A1 (design)**: verb naming — `value compare` vs domain-neutral
  `doctrine compare`. CLI-is-source-of-truth; settle before implementation.
- **OQ-A2 (design)**: implicit-session mechanics for ad-hoc rows (lock on a
  per-day file vs session-of-one files; merge-cleanliness is the driving
  force, RFC-019 T2).
- **OQ-A3 (design)**: does the tombstone-append verb (`withdraw`) ship in
  Phase A (capture-side, cheap) or wait for Phase B resolution semantics?
- **Assumption**: frame vocabulary for the value domain is the RFC-019 closed
  set; frames map to constraints only in Phase B, so Phase A stores the frame
  string and validates membership, nothing more.
- **Risk (low)**: schema churn when Phase B lands — mitigated by schema
  version field and RV-260 F-4's already-settled ordering key
  `(date, session_uid, row_seq)`.

## Verification / closure intent

- Round-trip tests: capture verb → session file → list verb, all row fields
  preserved verbatim (losslessness).
- Admissibility tests: record pair refused with reason; value-bearing pair
  admitted; RSK per REV-022 adjudication.
- Determinism: row uid/date injected via the date/uid pattern (no clock/rng
  in the pure layer); golden session-file fixture.
- Append-only invariant: capture never rewrites existing rows or files from
  prior sessions.
- No scoring diff: priority suites pass unchanged (behaviour-preservation
  gate on shared machinery).

## Summary

Ships the evidence pipe: typed, lossless, append-only pairwise judgements in
merge-clean session files, with admissibility enforced at capture — the
substrate Phases B–E consume, useful immediately as a durable record of
stakeholder orderings.

## Follow-Ups

- Phase B slice: row-validity resolution + constraint layer + projection +
  REV-022 apply (ADR-015 amendment).
- Phase C slice: elicitation queue.
