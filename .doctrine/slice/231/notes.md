# Notes SL-231: Friction observation ledger and capture interface

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-26 · design · 6393a93e

### Produced

- SL-231 — observation architecture designed and adversarial findings integrated
  (commits 1ff203ad..52e8f654)
- PRD-018 and SPEC-028 — draft observation capability and ledger-container
  contracts authored with REQ-397..REQ-413 (commit 6393a93e)
- REV-035 — SPEC-003 container inventory revised for SPEC-028 (commit 6393a93e)
- IMP-319 — subprocess-worker observation capture parity
- IMP-320 — configurable observation guidance in boot context
- quick check passed; full gate not run for this governance-only unit

### Learned

- DEC-043 — observations use a dedicated capability and container pending
  evidence for a reusable ledger abstraction

### Open

- QUE-176 — trustworthy per-harness token instrumentation boundaries
