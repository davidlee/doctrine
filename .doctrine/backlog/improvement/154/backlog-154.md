# IMP-154: search --all: index non-entity docs + path column

## Context

`doctrine search` indexes only catalog **entities**: `scan_catalog` →
`entity_lex_doc` builds each `LexDoc` from the entity's title + body (the entity
`.md`). Loose per-slice / per-adr markdown that is NOT an entity body —
`design.md`, `notes.md`, `plan.md`, `handover.md`, reconciliation briefs, etc. —
never enters the corpus, so their prose is unsearchable.

That prose is often where the real detail lives (design rationale, gotchas,
phase notes). A user grepping for a remembered phrase from a design doc gets no
hit.

## Proposed

1. A `--all` flag (name TBD — could clash with the kind-group `all`; consider
   `--docs` / `--full` / `--include-docs`) that widens the corpus to include
   non-entity markdown under the doctrine tree alongside the entity bodies.
2. When doc-scope is on, results are no longer all entity-addressable, so add a
   **Path / File column** to the table so a doc hit is locatable (entities show
   their id as today; doc hits show the relative path).

## Open questions

- Which files are in scope? (slice/adr dirs only? whole `.doctrine/`? runtime
  state like `handover.md` is gitignored/disposable — probably exclude.)
- Hit identity: an entity hit keys on its id; a doc hit keys on a path. Does the
  table show both columns always, or only add Path under `--all`?
- Ranking: mixing entity bodies and loose docs in one BM25 corpus — does doc
  length skew scores? May want per-source normalisation.
- JSON shape: add a `path` field / a `source: entity|doc` discriminator.

## Notes

Pairs with the IMP-153 table/`--context` work (comfy-table render, snippet
band) — the Path column rides the same `SEARCH_COLUMNS` mechanism.
