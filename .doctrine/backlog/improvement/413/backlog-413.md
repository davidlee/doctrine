# IMP-413: Advisory validation for unfilled knowledge facets

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`IMP-403` lead 3, minted at `SL-249`'s close. `SL-249` closed leads 1 and 2 —
`knowledge edit` writes the structured tier and the design-run wire mints a
filled record — and named 3–5 as follow-ups rather than dropping them.

## What

A `decision` may hold `status = "accepted"` with an empty `choice` and
`doctrine validate` reports clean. Nothing anywhere says a settled record owes
its deciding fields, so the corpus's 24% facet fill (measured on `IMP-403`,
2026-08-05) is invisible to every check.

## Shape

An **advisory** — a warning, not an error. `SL-249` shipped `doctor` check #12
(a `[facet]` key inert at its record's kind), so the check registry is the
obvious host: an unfilled-facet advisory sits beside it and reuses its
`Finding` category plumbing. The kind-dependence is real — `ConceptFacet {}`
has no fields by design, so "unfilled" is not a uniform predicate across the
seven kinds and the advisory must derive its expectation from
`knowledge::facet_fields`, the declaration table `SL-249` `PHASE-03` shipped.

Open: whether the advisory keys off *settled* status (an `accepted` decision
owes `choice`; a `proposed` one does not) or off mere existence.

## Related

`IMP-403` (parent), `IDE-009` (a knowledge lint verb — likely the same
mechanism at a different surface), `IMP-414` and `IMP-415` (leads 4 and 5),
`SPEC-019` (the per-kind facet contract, extended to seven kinds by `REV-050`).
