# Implementation Plan SL-204: Extract per-kind validation contract to break integrity coupling

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases, each ending green, ordered by a hard dependency chain: the
scaffold fn must leave `Kind` before the statics can leave the kind modules;
the statics must live in the leaf before the `KINDS` table can reference them
there; the table and resolvers must be in the leaf before `integrity` can drop
its 13 kind imports and the layering re-tier can be measured.

## Sequencing & Rationale

- **PHASE-01 (seam)** is the only phase touching *behaviour-adjacent* code —
  the `materialise` signature. It deliberately moves nothing between modules,
  so a failure isolates to the seam change. The three panic-stub scaffolds die
  here because the field forcing their existence dies here.
- **PHASE-02 (relocation)** is pure motion: `Kind` (now plain data) + statics
  into `kinds/mod.rs`, with re-exports pinning the blast radius (~200 field
  accesses untouched, design D4). The governance/RFC dual surface (leaf
  identities + in-module `GovKind` descriptors, RV-261 F-2) lands here.
- **PHASE-03 (retarget)** moves the table + resolvers and does the wide-but-
  mechanical consumer sweep (~98 refs, 26 files) in one compiler-enforced
  pass. Splitting it from PHASE-02 keeps "moving identity" and "moving the
  consumers of identity" independently revertable.
- **PHASE-04 (measure)** does no source motion — only the layering map,
  empirical tangle measurement, and the ratchet. Numbers are deferred to this
  phase by design D6 (SL-203 precedent: predicted deltas were wrong twice).

R1 (sprawl) is mitigated by the phase boundaries themselves; every phase is
independently green and committable. R2 (new leaf edges) is checked in
PHASE-04 by the gate delta, not integrity's local count.

## Notes

- The design's grep-pable premises were verified against the tree in the same
  session the design locked (integrity.rs import block, governance.rs run_new,
  entity.rs:374/477 scaffold invocations, kinds.rs vocabulary).
- Conformance target set = the 43 `design-target` selectors seeded at design
  time (`doctrine slice selector list SL-204`).
