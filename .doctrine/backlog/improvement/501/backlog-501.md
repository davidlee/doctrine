# IMP-501: Observation ledger: rank it in doctrine search and make recurrence countable

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

The observation ledger's read surface is deliberately weak, and the capture path
does not compensate for it.

- `doctrine observation search` is a **Boolean** matcher — every query token must
  appear somewhere in the envelope's combined text — with newest-first ordering and
  no scoring (`src/observation/query.rs::lexical_match`). `DEC-030` states the
  position: "Lexical matching is deliberately retrieval-oriented rather than a
  promise of ranked full-text search."
- `doctrine search` (SL-141) *is* ranked — BM25 over entity bodies — but its corpus
  is `scan_catalog` numbered entities. Observation records are UUID-addressed and
  structurally outside `ALL_KINDS`, so a search that reaches every entity reaches no
  observation.
- Net effect: finding prior friction requires knowing a second verb exists, and
  accepting a weaker matcher when you use it. Nothing in the capture path asks
  "has this already been recorded?".

The cost is visible in the ledger itself. Diagnosing the `ISS-465` memory-search
defect produced **five near-duplicate friction records plus one issue** for one
defect. A separate defect (observation summaries naming backlog ids whose titles are
unrelated) produced two byte-identical records. Both clusters were found by reading
the whole ledger by hand, not by a query.

`DEC-032` already models recurrence correctly: a later observation is a *separate
immutable* observation that correlates to its subject
(`CorrelationFacet.related_observations`). So N records of one friction are, in the
data model, N corroborations — the count is derivable today. There is simply no way
to see it, and no way to record a corroboration *as* one rather than as a fresh
report.

## Desired outcome

1. **Observations enter `doctrine search`'s corpus**, BM25-ranked alongside entities,
   so one ranked query reaches prior friction. `IMP-154` proposes widening that
   corpus to non-entity markdown; observation records are a second, differently-keyed
   source that belongs in the same widening.
2. **A derived recurrence count** ("updoot"): how many active observations share a
   subject, surfaced on read. Derived, never authored — the corpus stays immutable and
   per `DEC-030` no derived index becomes a second source of truth.
3. **Capture-path guidance**: search before recording; on a hit, corroborate rather
   than duplicate. This is the `IMP-320` prompt-fragment seam.

The point is to take the toil out of analysis: today, "which frictions recur, and
how often?" is a hand-read of 693 records.

## Open questions

- **Hit identity.** An entity hit keys on its prefixed id; an observation keys on a
  uid. `IMP-154` already has to solve the id-vs-path version of this; a third key
  shape makes the results table's identity column a real design question.
- **Subject link.** Is `related_observations` sufficient to express "this is the same
  friction", or does corroboration want a distinct edge with its own semantics
  (DEC-030's immutability rule forbids patching the subject)?
- **Governance boundary.** `DEC-030` excludes "aggregation, counts or grouping" from
  V1 and assigns them to "a reporting and analysis concern for a follow-up
  capability". Does a *derived read-time* count stay inside that boundary, or does
  this item become the follow-up capability and need its own decision — possibly a
  Revision of `DEC-030`? `SPEC-028`'s responsibility text ("without embedding
  aggregation or reporting policy") is the other surface to reconcile.
- **Ranking integration.** Reuse `Bm25Ranker` for the observation corpus, or does the
  envelope's payload/facet text need its own field weighting? `IMP-330` (one shared
  flatten struct across observation list/search args) is the adjacent DRY seam.

## Notes

Seams: `src/search.rs` (corpus build, `KindSelector`), `src/observation/query.rs`
(`lexical_match`, unranked), `src/observation/wire.rs` (`CorrelationFacet`),
`src/lexical.rs` (`Bm25Ranker`, shared).

Neighbours: `IMP-154` (widen the search corpus), `IMP-320` (capture guidance),
`IMP-330` (shared flatten struct). Governing: `PRD-018`, `SPEC-028`, `DEC-030`,
`DEC-032`.
