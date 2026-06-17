# Implementation Plan SL-092: Inspect sort + scan robustness

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML.

## Overview

Two mechanical fixes to the inspect/scan pipeline, both surfaced in the SL-046
code review and bundled here because they touch the same module neighbourhood.

## Sequencing & Rationale

**PHASE-01 (numeric inbound sort)** comes first. It is a single ~5-line change
in `inspect_from` with one test extension. No signature changes, no call site
updates, no risk to downstream consumers. Quick green/merge: a clean warm-up
that proves the test harness before the more invasive PHASE-02.

**PHASE-02 (graceful scan degradation)** follows. It changes the `scan_entities`
signature (`&mut Vec<CatalogDiagnostic>` parameter) and touches 4 non-test call
sites plus test fixture updates. The behaviour-preservation gate is the
existing suite — it must stay green unchanged for a well-formed corpus. New
tests prove the skip path for malformed entities.

Both phases are file-disjoint in the implementation body (PHASE-01 only touches
`relation_graph.rs`; PHASE-02 touches `catalog/scan.rs`, `catalog/hydrate.rs`,
`main.rs`, `priority.rs`, and `relation_graph.rs`'s test wrapper). However,
PHASE-02's EX-1 depends on PHASE-01 being complete — the existing suite must be
green before the scan signature changes. Sequential execution is correct.

## Risk

Low. Both changes ride existing infrastructure (EntityKey::Ord, CatalogDiagnostic).
No new dependencies, no schema changes, no stored data. The existing test suite
is the behaviour-preservation gate for the well-formed corpus path.

## Notes

- The `--strict` flag (deferred out of scope) and the read-amplification
  reduction are separate improvements — see `slice-092.md` § Non-scope.
- The queried entity's own parse failure remains a hard error via the F6
  existence gate — this is by design.
