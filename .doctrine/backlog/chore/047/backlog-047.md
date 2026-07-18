# CHR-047: Claims-era render/allowlist residue sweep

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Deliberate residue from SL-222 PHASE-09 (RV-284 F-7; slice notes "Deferred /
known residue"):

- `src/priority/render.rs`: `CostUnmigratedFacet => (0.0, 0.0)` and
  `ValueUnmigratedFacet => (0.0, MARKER)` arms are unreachable-by-construction
  — the parse-free tripwire mints no magnitude. Collapse or justify them.
- NF-001 facet-substring allowlist entries were refreshed during SL-222 but a
  minimality pass was never enforced — prune to the entries the claims-era
  code actually exposes.
- `src/estimate.rs` / `src/value.rs` retain `deserialize_lenient` solely for
  serde round-trip on doc structs — check whether the post-deletion doc
  structs still need it.

No behaviour impact (SL-222 audit ran suite + gate green with the residue in
place); this is shape cleanup only.
