# IMP-121: Unified corpus health doctor verb

## Context

Corpus integrity checking is fragmented across at least four disjoint command
surfaces, with no single "is my governance graph healthy?" gate:

- `doctrine validate` — id-integrity only
- `spec validate` — FK integrity, dangling members/interactions, orphan
  requirements
- `memory validate` — dangling relations, stale verification, draft expiry
- relation/supersession integrity — per-entity danglers via `inspect`

Each is real and good; the gap is that there is **no aggregating surface**. A team
or CI job that wants "is the corpus sound?" must know and run four+ separate
commands, union their exit codes, and reconcile four output shapes.

## Recommendation

One `doctrine doctor` (or `validate --all`) that runs every integrity check across
the whole graph — id, FK, danglers, orphans, supersession, memory — and returns one
go/no-go plus a unified, actionable report.

## Sub-checks in scope

- **Done-but-open detector** (proposal 0026): advisory flag for open backlog items
  whose linked slices are all terminal. Pure graph query over item→slice edges;
  advisory only, never auto-close.
- **Prose citation integrity** (proposal 0029): extract every `KIND-NNN` citation
  from authored `.md`, report unresolved ones. Reuses the existing prose scan
  primitive (`integrity.rs:518`) currently only invoked by `reseat`. Advisory only;
  precision-exclude code spans, sentinels, and doc-local refs.

_Source: proposals 0011, 0026, 0029 (loop/proposals-2026-06-20), 2026-06-20._
