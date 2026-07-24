# Notes SL-204: Extract per-kind validation contract to break integrity coupling

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-24 · PHASE-04 concluded · 37ae03bc (re-tier) · f5a40ac0 (boundary)

### Produced
- kinds/{mod,resolve}.rs own KindRef + KINDS + the 5 ref probes + `canonical_id`;
  integrity's 13 kind-module up-reaches gone, the 4 back-cycles broken.
- PHASE-04: integrity re-tiered command→engine; command tangle ratcheted **99→76**
  (measured empirically — integrity was entangled with the core SCC, so ejection
  drops −23, not the 4 nominal 2-cycles; RSK-227/SL-203 precedent).
- canonical-id single-sourced: `kinds::resolve::canonical_id` is the format
  authority; `listing::canonical_id` delegates down (caller seam preserved, zero
  retarget); resolve.rs dropped its PHASE-03 inline (D2 follow-up, resolved).

### Learned
- **Design D2 premise was false.** D2 mandated `kinds/resolve.rs` route id-formatting
  through `listing::canonical_id`. Infeasible: `listing → tag` (listing.rs:222) and
  `tag → kinds` (tag.rs:15 re-exports `kinds::TAGGABLE`), so `kinds → listing` closes
  a leaf 3-cycle. Resolved in PHASE-04 by relocating the authority DOWN into `kinds`
  (below tag) and delegating `listing → kinds`.
- **vt9 gate-blocker (ISS-235).** `worker_commit`'s full-suite gate false-red on
  `memory::…::vt9_no_discoverable_root_emits_nothing` — non-hermetic on ambient
  `CLAUDE_PROJECT_DIR` + live corpus, orthogonal to the delta. Fixed hermetic
  (rootless tempdir cwd) inside the PHASE-04 worker delta so the gate passes;
  `src/memory.rs` declared a collateral selector. RFC-011 case-notes
  `SL204-a15d-P04-vt9-gate-falsered`.

### Re-tier verdicts (EX-3 / VA-B — Tier-3 follow-up input)
- **catalog: STAYS command.** Not freed by integrity's re-tier. `catalog::scan`
  reaches 9 command kind-handlers (adr, backlog, knowledge, memory, policy, rec,
  review, revision, slice, spec, standard) to hydrate/scan every kind — its command
  coupling is independent of integrity. A Tier-3 follow-up needs its own
  contract-extraction (the SL-204 pattern, larger scale).
- **relation_graph: STAYS command.** Its all-kind seam imports adr, backlog,
  catalog, governance, policy, spec, standard (all command) directly — integrity
  (now engine) was only one of many command reaches, so re-tiering it changes
  nothing here. Own extraction required; not unblocked by SL-204.

### Open
- (none for SL-204 — all phases landed; catalog/relation_graph extractions are
  separate future Tier-3 slices, not SL-204 scope.)
