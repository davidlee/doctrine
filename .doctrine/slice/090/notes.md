# Notes SL-090: Wire link/unlink CLI and template for memory relations

## 2026-06-17 — Inquisition complete (RV-059)

RV-059 tried the design on 8 charges. All resolved. Key findings:

- **F-1 (blocker): D4 contradicts scope.** Scope says "items/ first, shipped/
  fallback"; D4 tightened to "items/ only" with no delta annotation. Must be
  reconciled before /plan.
- **F-2 (design-wrong): Shipped/ clone dead end.** "clone to items/ first"
  prescribes an unavailable verb. Fix depends on F-1 resolution.
- **F-4 (major): D6 fork.** Design must commit to duplicating F1 guard in
  memory.rs — RelationLabel is vocabulary-bound; memory labels are raw.
- **F-3/F-5/F-6/F-7/F-8 (minor/nit):** Import direction, test disambiguation,
  path joiner, help text, stale handover — all straightforward fixes.

The design is salvageable with small corrections. See RV-059 synthesis for the
ordered penance.

Durable pattern recorded: `mem.pattern.link.memory-label-fork` — memory write
path labels are free-form; RELATION_RULES vocabulary is closed to memory edges.
