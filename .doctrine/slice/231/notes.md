# Notes SL-231: Friction observation ledger and capture interface

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · PHASE-01 delivered, pending import · 3e2733b0

### Produced

- PHASE-01 — typed observation core delivered on the pi/deepseek arm; committed
  on `dispatch/SL-231-p01` @ e009c0f7b (duplicate `w/SL-231-p01` @ 6936a3ab5,
  trees provably identical). NOT yet imported, verified, or VT-gated.
- ISS-260 — ADR golden worker-marker skip read only the env leg; fixed on edge
  (9cd0e7706) and merged into `dispatch/231`
- IMP-328 — pi spawn fork is unbound to slice/phase, so a pi-arm fork is
  unresolvable by the funnel import resolver
- `scripts/pi-spawn-confined.sh` — PI_REUSE_FORK / PI_THINKING / PI_TOOLS env
  overrides (e2dbf0198); parity scraper re-verified
- dispatch coordination for SL-231 established; ADR-001 `observation = "leaf"`
  pre-seeded on the coord branch (095fca404) because `.doctrine/` is a worker
  forbidden zone
- SL-231 — five-phase executable plan authored, critically strengthened,
  materialised, reviewed, and advanced to ready (commits aee493b2..2baca05d)
- post-ready plan review — removed brittle line anchors and broad selectors,
  aligned scope exclusions, and carried REQ-412 purity verification through
  store embedding and final-state verification
- PHASE-01 through PHASE-05 — runtime sheets materialised under
  `.doctrine/state/slice/231/phases/`
- IMP-322 — make Pi research runners tolerate read-only session homes
- pre-design research re-baselined after orchestrator fallback; both mandated
  Pi producers failed before repository inspection on read-only `/home/david/.pi`
- quick check passed with repository-pre-existing warnings; full gate not run
  for this governance-only planning unit

### Learned

- PHASE-01's EX-5 is not worker-satisfiable: `architecture_layering_gate` raises
  `Unclassified` for any `src/` unit absent from `.doctrine/adr/001/layering.toml`,
  which workers may not write. Orchestrator must pre-seed the classification onto
  the coord branch before forking. Same shape applies to PHASE-05's
  `.doctrine/governance.md` + `rfc-011.md` (orchestrator-authored).
- `warnings = "deny"` + `unused = "deny"` + `allow_attributes = "deny"` make a
  consumer-less new module uncompilable; the one sanctioned escape is a single
  module-level `#![expect(dead_code, reason = …)]`.
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
- IMP-328 — pi spawn fork unbound to slice/phase (blocks funnel import on the pi
  arm until the script derives branch/dir from `(slice, phase)`)
- PHASE-01 import → verify → verify-vt → conformance not yet run; the phase's
  green claim rests on its own suite (42/42 observation) plus a worker-tree gate
  that carried the then-unfixed ISS-260 reds
