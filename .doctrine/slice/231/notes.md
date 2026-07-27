# Notes SL-231: Friction observation ledger and capture interface

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · PHASE-01 landed and reaped · addc6d17

### Produced

- PHASE-01 — typed observation core delivered on the pi/deepseek arm and driven
  through the full funnel: imported (`1d8cc08ae`), verified green on the `gate`
  cadence (`02da8ebf4`), concluded (`addc6d178`), fork reaped. Boundary row
  `[0d2cb5671, 1d8cc08ae]` recorded.
- PHASE-01's green is now INDEPENDENTLY established, discharging the delivery
  caveat: the unmarked coord-tree gate passed, and `check regression diff`
  against the B baseline showed no new or changed failures. The single baseline
  failure at B — `architecture_layering_gate` raising `StaleEntry("observation")`
  because the ADR-001 leaf entry was pre-seeded ahead of the module — flipped to
  `fixed` by the import, which is the mechanical proof that EX-5's leaf
  classification is live.
- ISS-263, ISS-264 — two dispatch-funnel defects found while landing PHASE-01
  (opaque import fault on a bad fork name; conformance `--strict` false positive
  on the import-landed funnel row).
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
- `slice verify-vt 231` is a SLICE-level conclude-cadence gate and remains
  pending until all five phases land — not a PHASE-01 gap.
- PHASE-01 conformance is substantively clean (6/6 declared source paths hit);
  its one `--strict` failure is the ISS-264 machinery false positive, not a
  worker scope violation.
- No ledgered code review exists for the PHASE-01 delta. The informal deepseek
  pass produced one confidently-wrong finding and silently skipped the check
  flagged as most important, so it is not sufficient evidence on its own. VT-4
  was adjudicated against the landed code instead: all six purity categories are
  pinned and the leaf classification is enforced by the real gate. The residual
  is that no observation-SPECIFIC upward-dependency negative test exists — the
  gate's generic rejection tests cover the mechanism.
