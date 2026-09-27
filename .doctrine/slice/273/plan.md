# Implementation Plan SL-273: Library ownership and lib: citations

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Four phases, as DEC-346 fixed them: build the check, settle the owners, list
every occurrence, apply the list. The check comes first so that every `lib:`
citation written afterwards, by hand or by a worker, is held to resolving from
the moment it lands.

## Sequencing & Rationale

- **PHASE-01 — resolver and check (Claude, TDD).** No corpus edits. The
  resolution test is enforcing from day one (vacuous until citations exist);
  the bare-mention test ships `#[ignore]` and serves as the sweep's working
  list. The shipped-roots walk is one function taking a root and a manifest, so
  the RV-408 F-2 and F-4 controls can seed it with a fixture root and a
  manifest missing an address. The F-3 and F-5 controls are core and doctor
  unit tests.
- **PHASE-02 — teaching and consolidation (Claude).** Lands before the
  inventory because the rename and the `boot-footer.md` retirement move
  citation targets (R2). The Revision is approved by the user and applied
  before the rename commit, so governing text never names a file that does not
  exist. Sequence inside the phase: Revision drafted → user approval → apply →
  rename + manifest + boot → owner-map cuts and teaching → memory repair →
  old-name sweep. Shipped-memory edits are committed as soon as they are
  synced (ISS-497, concurrent-agent reversion).
- **PHASE-03 — inventory (DeepSeek, read-only; orchestrator adjudicates).** The
  enumeration is read-only, so it runs through `./scripts/pi-research` with its
  stdout captured, not a confined dispatch: the output is authored `.doctrine/`
  state, which the orchestrator alone writes. The ignored bare test gives a
  machine completeness control over the test roots; the `src/**` literals have
  no machine control and rest on the worker's enumeration plus the audit
  re-pass.
- **PHASE-04 — apply and enforce (DeepSeek, confined dispatch).** The
  per-occurrence verification script is written and proven against the RV-408
  F-1 control *before* the worker is dispatched, then run on the real hand-back
  before the import is accepted. Un-ignoring the bare test is the act that
  makes I2 enforcing.

**RV-408 routed findings.** F-1 to F-5 were reopened to add `control` routes
and stay open until their controls exist. Each is transcribed on its host
phase, citing the finding in the criterion text: F-2, F-3, F-4 on PHASE-01's
EX-6..EX-8, F-5 on PHASE-01 EX-9, F-1 on PHASE-04 EN-2 and EX-2. Codex
verifies them on the ledger once the phases land.

**After the plan.** `/audit` runs a second DeepSeek pass with the PHASE-03
brief against the landed tree (`inventory-audit.toml`, design 6.4); that is
audit's work, not a phase.

## Notes

- A handover before PHASE-03 is expected; phases 3–4 run on DeepSeek workers.
- If PHASE-02 proves too large for one sitting, the natural split is the
  Revision + rename + repair versus the owner-map cuts and teaching. Splitting
  changes DEC-346's four-phase shape, so it goes to the user first.
