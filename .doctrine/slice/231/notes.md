# Notes SL-231: Friction observation ledger and capture interface

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · design · 32b9fdd3

### Produced

- SL-231 — design and scope reconciled through the adversarial re-review
  (commit 32b9fdd3)
- RV-311 — nine re-review findings adjudicated
- REV-037 — observation worker, enrichment, correction, query, and publication
  contracts reconciled
- REV-038 — SPEC-013 member requirements aligned with its non-entity ledger
  exception
- quick check passed; full gate not run for this governance-only unit

### Learned

- DEC-044, DEC-045, DEC-046, DEC-047, DEC-048, DEC-049, DEC-050, DEC-051,
  DEC-052 — UUID identity, correction, publication, capture, query, enrichment,
  safety, and authored-storage contracts
- EVD-002 — `claude -p` is the first candidate for trustworthy token telemetry
- RV-311/F-1 — marked solo worktrees defer friction for coordination-tree capture
- RV-311/F-5 — source admission is fixed before the harness extraction adapter,
  which remains owned by QUE-176
- RV-311/F-6 — SPEC-013 owns the explicit non-entity ledger exception

### Open

- QUE-176 — trustworthy per-harness token instrumentation boundaries
- IMP-319 — subprocess-worker observation capture parity
- IMP-320 — configurable observation guidance in boot context
- IDE-005 — harness identification through bounded environment enrichment
