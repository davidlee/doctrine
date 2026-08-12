# IDE-050: Definition records for ubiquitous language

Precisely define a **term** — in the ubiquitous-language sense — whose shared
understanding within a given context is load-bearing for correct comprehension.
Not a concept to be reasoned about; a word to be *agreed on*.

## Why

Surfaced in the `RFC-025` capsule programme, where **admission** turned out to
name two unrelated events, and *verify* and *conform* each name two more:

| word | work axis | mechanism axis |
|---|---|---|
| admission | `ADR-020` step 5 — the control plane advances the accepted ref by compare-and-swap | `AdmissionVerdict` — every conformance row over a *backend* is `Proven` |
| verify | `ADR-020` step 4 — verify the normalized candidate in a fresh capsule | `backend verify` — run the property suite |
| conform | `ADR-020` step 4 — conform the pinned result to its contract | `conformance.rs` — the property suite itself |

The cost was not hypothetical: two participants held different readings of
*admission* through a whole exchange, and a retention decision (`DEC-193`) was
drafted against the wrong event because of it. Neither reading was wrong — the
word was.

No existing kind carries this. `CPT` (concept) is the nearest and is a
different job: a concept is a *model* argued for in prose (`CPT-002` is an
essay on threat shape), whereas a definition is a short binding of a term to a
meaning within a boundary. Filing definitions as concepts would dilute both.

## Two candidate paths, both subject to design scrutiny

1. **Extend `concept`** with facet fields — e.g. `term` / `name` and
   `definition`. Cheap: `concept` presently has *no* facets at all
   (`knowledge edit concept` exists only to refuse), so there is a slot. Risk:
   conflates two jobs under one kind and one id namespace.
2. **Mint a new kind** — e.g. `definition` / `DEF-NNN`. Clean separation and
   its own namespace; costs a kind, with everything a kind drags (templates,
   list/show/inspect wiring, retrieval, glossary entry).

## What matters whichever path wins

- **The bounded context must be first-class.** A definition is only true inside
  a boundary; the same word legitimately means something else elsewhere. The
  record has to *represent* that scope, not gesture at it in prose — otherwise
  it becomes a global claim and reintroduces the collision it exists to prevent.
- **Retrieval must be matched to that bound.** Triggers and search scope should
  fire when a reader is inside the context and stay quiet outside it. A
  definition that surfaces everywhere is noise; one that surfaces nowhere is
  ceremony. This is the harder half of the design.
- Relationship to `glossary.md` needs deciding — the shipped glossary already
  defines doctrine-wide terms, so a definition record is presumably for
  *project- and context-local* language, with the glossary as the global tier.

## Provenance

`RFC-025` capsule programme, 2026-08-12, on the exchange that produced
`DEC-193`'s rescoping.
