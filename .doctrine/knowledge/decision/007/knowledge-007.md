
SL-226 Q1: the CLI graph emitter is a new top-level `doctrine graph` verb with
`--format dot|json`, not an extension of `catalog graph` (debug-tier) and not
dot-only. `--format json` serializes the filtered/bounded subgraph through the
same `CatalogGraph` serde contract that `catalog graph` and `/api/graph` use —
shared serialization, no parallel payload shape. `catalog graph` remains the
whole-corpus debug dump; supersede later only if it proves redundant.
Rationale: first-class consumption surface at PRD-016 altitude (RFC-001);
overlap cost is one serde call.
