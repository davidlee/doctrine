# IMP-034: Interrogate refactoring all relations modelling to a uniform schema across kinds

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

Interrogate whether **all** relations modelling should be refactored to a single
uniform schema, instead of today's bespoke per-kind typed fields. This is an
**interrogation/decision** task (likely landing an ADR or a `/spec-tech` decision),
not a pre-committed refactor.

## The variety, today (surveyed during SL-046 design)

- Three independent **private `struct Relationships`** (slice / governance /
  backlog), plus spec lineage modelled differently again: `descends_from`/`parent`
  as `Option<String>` on the spec `Meta`, and `members[].requirement` in a separate
  `members.toml`.
- Mixed **cardinality** (`Vec` multi vs `Option` single) and mixed **structure**
  (plain refs vs `members.{label,order}` vs `after.rank`).
- Mixed **aliveness** — slice/governance `[relationships]` are parsed-but-inert;
  backlog's are live.
- A stored reciprocal (`superseded_by`) that violates ADR-004 (tracked separately
  as IMP-032).

## The question to settle

Is a single generic surface — e.g. `[[relation]] kind="…" target="…"` (with an
optional attributes table for ranked/labelled edges) — across **every** kind worth
it, vs the ergonomics of bespoke typed fields (`slice.specs` reads better than a
generic array)? Trade-offs to weigh: uniform tooling/validation/rendering and one
extraction seam, vs per-kind readability and migration cost across the whole corpus.

## Boundaries / sequencing

- **Not SL-046.** SL-046 is the reader; it *normalises* the existing variety behind
  a per-kind `relation_edges` accessor (legitimate variety stays, the adapter
  unifies the read). This item asks whether the **storage** itself should unify.
- Run **in parallel with, or as a direct successor to, SL-046**, and likely **feed
  the relation-governance ADR that SL-048 needs** (uniform schema decisions and the
  cross-corpus capture surface are entangled).
- Adjacent: IMP-006 (uniform destructive/lifecycle verbs), IMP-016 (cross-corpus
  reference links), IMP-032 (superseded_by → derived).

Related: [[SL-046]] · [[SL-048]] · IMP-006 · IMP-016 · IMP-032 · ADR-004.
