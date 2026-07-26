# Notes SL-231: Friction observation ledger and capture interface

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · ready · 604bae5c

### Produced

- SL-231 — five-phase executable plan authored, critically strengthened,
  materialised, and advanced to ready (commits aee493b2..604bae5c)
- PHASE-01 through PHASE-05 — runtime sheets materialised under
  `.doctrine/state/slice/231/phases/`
- IMP-322 — make Pi research runners tolerate read-only session homes
- pre-design research re-baselined after orchestrator fallback; both mandated
  Pi producers failed before repository inspection on read-only `/home/david/.pi`
- quick check passed with repository-pre-existing warnings; full gate not run
  for this governance-only planning unit

### Learned

- DEC-044, DEC-045, DEC-046, DEC-047, DEC-048, DEC-049, DEC-050, DEC-051,
  DEC-052 — UUID identity, correction, publication, capture, query, enrichment,
  safety, and authored-storage contracts
- EVD-002 — `claude -p` is the first candidate for trustworthy token telemetry
- RV-311/F-1 — marked solo worktrees defer friction for coordination-tree
  capture

### Open

- QUE-176 — trustworthy per-harness token instrumentation boundaries
- IMP-319 — subprocess-worker observation capture parity
- IMP-320 — configurable observation guidance in boot context
- IDE-005 — harness identification through bounded environment enrichment
