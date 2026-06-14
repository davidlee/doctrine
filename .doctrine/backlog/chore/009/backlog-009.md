# CHR-009: Coordinate SL-068 candidate-workflow orientation masters into SL-069 cohesive corpus

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

SL-068 PHASE-07 ships the candidate workflow but deliberately authors **only the
mechanical** global memory update (the `mem.signpost.doctrine.cli-command-map`
verbs). The **freeform orientation masters** a doctrine client needs to *use* the
candidate workflow — the workflow narrative, the admission-by-OID invariant, and
the SL-067 evidence-ref-is-not-a-branch trap — are deferred here so SL-069
("Shipped memory corpus as a cohesive client onboarding anchor", governed_by
ADR-002) can author them as part of one curated corpus rather than SL-068
sprinkling ad-hoc masters into a corpus SL-069 is about to make coherent.

Why a backlog item and not a structural edge: a slice→slice `after`/`needs` edge
is not authorable with current verbs (`needs`/`after` are slice→work; `link` has
no relate-label legal for SL — only specs/requirements/supersedes/governed_by).
This item is the durable coordination record until SL-069 is scoped.

Action when SL-069 is designed: fold the candidate-workflow orientation into its
corpus scope; confirm SL-068 did not also ship those masters (avoid duplication).

Cross-ref: SL-068 PHASE-07 (EX-4), SL-069, ADR-002.
