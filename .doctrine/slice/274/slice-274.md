# Ranked search and derived corroboration for the observation ledger

## Context

The observation ledger (`PRD-018`, `SPEC-028`, SL-231) now holds ~693 records.
Its read surface is deliberately unranked: `doctrine observation search` is a
Boolean all-tokens-must-match filter over the envelope's combined text, with
newest-first ordering and no scoring. `DEC-030` states the position — "lexical
matching is deliberately retrieval-oriented rather than a promise of ranked
full-text search". `doctrine search` (SL-141) *is* ranked (BM25) but its corpus
is `scan_catalog` numbered entities; UUID-addressed observation records sit
structurally outside `ALL_KINDS`, so the ranked surface cannot reach them.

The cost is measurable in the ledger itself. Diagnosing the `ISS-465`
memory-search defect produced **five near-duplicate friction records and one
issue** for one defect; a second defect (observation summaries naming backlog ids
whose titles are unrelated) produced two byte-identical records. Both clusters
were found by hand-reading the ledger, not by a query, and nothing in the capture
path asks "has this already been recorded?".

`DEC-032` already models recurrence correctly: a later observation is a *separate
immutable* observation correlating to its subject
(`CorrelationFacet.related_observations`). N records of one friction are, in the
data model, N corroborations — the count is derivable from the corpus as it
stands. Only the ranked read and the count are missing.

Promoted from `IMP-501`.

## Scope & Objectives

1. **Ranked retrieval over observations.** Observation records join
   `doctrine search`'s BM25 corpus, so one ranked query reaches prior friction
   alongside entities.
2. **Derived corroboration count.** A read-time projection — how many active
   observations share a subject — surfaced where an observation is read. Derived,
   never authored.

These are one change: both need the same corpus identity answer and both hinge on
the same `DEC-030` boundary question. If design triage splits them (`R1`, `OQ-1`),
the ranked read ships first.

## Non-Goals

- **Capture-path guidance** ("search before recording"): `IMP-320` owns the opt-in
  `doctrine.toml` → boot-prompt fragment. This slice makes the read cheap; it does
  not teach the habit.
- **Any change to the write path** — envelope schema, facet shapes, storage
  layout, capture verbs. The corpus is read-only input here.
- **Authored counters.** Every number this slice produces is recomputed from the
  corpus. No derived index becomes a second source of truth (`DEC-030`).
- **Loose-document search** (`IMP-154`): design.md/notes.md/handover prose is a
  different corpus with a path identity, not a uid one. Adjacent, not this.
- **The `ISS-465` memory-search read defects** (default page, relevance floor) —
  same retrieval family, a different corpus and ranker call site.

## Affected surface

- `src/search.rs` — corpus construction, `KindSelector`, the per-entity `LexDoc`
  build; the source-selection seam.
- `src/observation/query.rs` — the Boolean matcher, to be kept as the collection
  interface or superseded by the ranked path (`OQ-3`).
- `src/observation/wire.rs` — `CorrelationFacet.related_observations`, the
  corroboration substrate.
- `src/lexical.rs` — `Bm25Ranker` reuse; no new ranker.
- `src/listing.rs` — the results table's identity column (`OQ-4`).

## Risks & assumptions

- **A1** — the corpus is small enough (~693 records) that scan-and-rank is
  acceptable with no index, mirroring `DEC-030`'s "a query may scan it directly;
  any derived index is disposable".
- **A2** — `DEC-032`'s correlated-observation model is honoured in practice: the
  duplicate records in the ledger were recorded independently, not correlated, so
  the derivation may find *fewer* corroborations than a human would count. Confirmed
  by research before the count is specified.
- **R1** — identity mismatch: entities key on prefixed ids, observations on uids. A
  third key shape may force a results-table redesign (`IMP-154` carries the
  id-vs-path version of this).
- **R2** — `related_observations` was designed for correlation, not subject
  grouping. If it is populated for other purposes, a count over it over-counts.
- **R3** — corpus dir-walk hazards: the `NNN-slug` symlink alias beside each
  numeric entity dir, and the random-tail-sharded UUID observation tree.

## Open questions

- **OQ-1** — Does a derived read-time count stay inside `DEC-030`'s boundary ("V1
  does not provide aggregation, counts or grouping")? If it needs its own decision,
  that is a Revision (`ADR-013`) of `DEC-030`; `SPEC-028`'s "without embedding
  aggregation or reporting policy" is the second surface to reconcile.
- **OQ-2** — Does the observation corpus reuse `Bm25Ranker` unchanged, or does the
  envelope need per-field weighting (summary vs detail vs facets)?
- **OQ-3** — Is the Boolean collection interface kept alongside the ranked path, or
  replaced? Two matchers over one corpus is the duplication this repo bans.
- **OQ-4** — Hit identity in the results table: uid, subject, or a synthesised
  handle.
- **OQ-5** — Does this slice need `IMP-154`, or does the observation source land
  independently and leave `IMP-154` for loose documents?

## Verification / closure intent

- A ranked query over the observation corpus surfaces a prior friction record when
  phrased from a *different* symptom description than that record's own wording —
  the recall hand-reading currently supplies.
- The corroboration count for the duplicate cluster already in the ledger (the
  five `ISS-465` records) reads 5 from the derivation, not from a hand count.
- Behaviour preservation: the existing observation read suites stay green
  unchanged — the collection interface's semantics are a contract, not an
  implementation detail.

## Follow-Ups

Pre-design research round (`/research`) is the next act: it must establish how
`related_observations` is actually populated (`A2`/`R2`), the current
`kind_lex_doc`/`LexDoc` build shape in `src/search.rs`, and the `DEC-030` count
boundary's governance reach.
