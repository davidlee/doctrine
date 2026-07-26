# Notes SL-231: Friction observation ledger and capture interface

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · design · 81fdf237

### Produced

- SL-231 — design and scope reconciled after adversarial review (commit 1dd7b15d)
- RV-310 — thirteen adversarial findings adjudicated
- REV-036 — SPEC-028 and REQ-405..REQ-413 reconciled with the accepted design
  decisions (commit 81fdf237)
- quick check passed; full gate not run for this governance-only unit

### Learned

- DEC-044, DEC-045, DEC-046, DEC-047, DEC-048, DEC-049, DEC-050, DEC-051,
  DEC-052 — UUID identity, correction, publication, capture, query, enrichment,
  safety, and authored-storage contracts
- EVD-002 — `claude -p` is the first candidate for trustworthy token telemetry
- RV-310/F-6 — observations reuse lexical and rendering leaves without entering
  the entity-oriented CommonListArgs conformance spine

### Open

- QUE-176 — trustworthy per-harness token instrumentation boundaries
- IMP-319 — subprocess-worker observation capture parity
- IMP-320 — configurable observation guidance in boot context
- IDE-005 — harness identification through bounded environment enrichment
