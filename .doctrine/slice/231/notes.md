# Notes SL-231: Friction observation ledger and capture interface

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · PHASE-01..03 landed and reaped · 0fe9572b

### Produced

- PHASE-03 — CLI, reads, and corrections delivered on the pi/deepseek arm, then
  driven through the funnel: imported (`6a07967c2`), verified green
  (`ef7c9b455`), concluded (`0fe9572b`), fork reaped. Boundary row
  `[4163c554b, 6a07967c2]`. Conformance 11/11 declared source paths.
- PHASE-03 needed THREE orchestrator cleanup turns — more than P01/P02
  combined. Each finding was concrete and gate-invisible:
  1. `escape_hostile` iterated bytes and passed them through `char::from(u8)`,
     a Latin-1 mapping, so every multi-byte UTF-8 char was corrupted
     ("é" → "Ã©"). The suite was green only because every test string was pure
     ASCII. Now char-wise with C1 handling, pinned by
     `non_ascii_content_survives_rendering_intact`.
  2. The adapter hand-rolled `filter_and_order`, duplicating `query::query`,
     which already supports `Projection::History`. The tell: the worker
     annotated `Projection::History` as `expect(dead_code, "PHASE-04")` while
     hand-implementing history mode. Comparators were byte-identical, so the
     defect was latent drift, not a live bug. `filter_and_order` deleted; both
     paths route through the service.
  3. EX-5 was not fully discharged: an unescaped newline in a comfy-table cell
     let crafted content render as an apparent extra row. Fixed with ONE
     escaper taking an `EscapeContext` (Inline for cells, Block for detail) —
     not a second escaper, which would have repeated defect 2.
- Finding 3 came from asking the worker for its READ on an ambiguous point
  rather than mandating a fix. It identified the attack correctly but proposed
  an unworkable remedy (escape newlines always, re-apply formatting after —
  impossible once content newlines are indistinguishable from layout ones).
  Worth repeating the technique; not worth accepting the answer unexamined.

- PHASE-02 — no-clobber publication and store delivered on the pi/deepseek arm,
  then driven through the funnel: imported (`70f131d43`), verified green
  (`c699fd99b`), concluded (`7e6f7c0b`), fork reaped. Boundary row
  `[da66aa111, 70f131d43]`. Conformance 4/4 declared source paths; the single
  `--strict` failure is the ISS-264 machinery false positive.
- PHASE-02's fork was minted FUNNEL-BOUND up front
  (`worktree fork --slice 231 --phase PHASE-02 --worker` into
  `<coord>/.worktrees/SL-231-p02`) and the worker attached via `PI_REUSE_FORK=1`,
  so IMP-328 cost nothing this phase — no re-fork, no cherry-pick, and the
  import resolved first try.
- Orchestrator-directed cleanup turn on the PHASE-02 fork BEFORE the delta
  commit, fixing two defects a green gate cannot catch: (a) STD-001 — the
  reserved publication-temp prefix was a literal in `fsutil` and an independent
  private const in `store`, a silent corpus-corruption path if either drifted;
  now one `fsutil::PUBLICATION_TEMP_PREFIX` with a test crossing both code
  paths. (b) parallel implementation — `ensure_dir_components` was a near-verbatim
  copy of the extracted `ensure_parent_dirs`; both now route through one
  `create_dir_component` helper whose bool return preserves entity.rs's
  rollback contract. Fixed pre-commit deliberately: `record-delta --commit S`
  pins ONE commit's patch, so a post-import fix would fall outside the phase's
  boundary row.

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
