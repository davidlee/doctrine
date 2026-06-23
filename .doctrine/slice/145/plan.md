# Implementation Plan SL-145: Backlog relation source parity

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

One phase. The design (D1/D2/D3) reduced Axis A to a single legal-set widening in
`src/relation.rs`: add BACKLOG to the `sources` of `governed_by` and of the `related`
`[SL, RFC]` row. Both read and write seams are already kind-generic — `backlog::
relation_edges` returns `item.tier1` (all legal rows), and `run_link`/`append_edge`
carry no source-kind allowlist — so `validate_link` is the only gate, and widening the
table is the whole code change. The remaining work is test reconciliation (flip the
goldens that encoded the old refusal) and the user-elevated CLI end-to-end proof.

## Sequencing & Rationale

A single phase is the honest shape: the change is one coherent unit in one file,
behaviour-preserving for every existing edge, and splitting `governed_by` from `related`
would fragment a 4-line table edit and its shared test surface for no gain.

TDD order within the phase:
1. **Red** — flip/extend the unit goldens to assert the *target* behaviour: backlog
   `governed_by`/`related` legal and emitting (read_block golden at relation.rs:1429),
   the widened VT-2 `Related` expected set, and the new positive `validate_link` +
   target-gate-negative tests. These fail against the unwidened table.
2. **Green** — widen the two `sources` sets; update the VT-2 doc-comment prose so it
   stops asserting backlog emits only slices/specs/drift.
3. **Refactor** — confirm no incidental duplication; the change rides the existing
   `sources: SET` shape (ADR-010 D2), so there is nothing to extract.
4. **Behaviour-preservation** — run the full root suite; the ONLY churn permitted is the
   intentional refusal-golden flips (EX-6). Any other diff is a design defect, stop and
   re-examine.
5. **End-to-end (VH-1)** — on a fresh `./target/debug/doctrine` (never the stale RO jail
   binary, R3), exercise the real CLI loop against a live backlog item: link `governed_by`
   an ADR and `related` a numbered entity, confirm TOML persistence, `inspect` outbound +
   derived inbound, and `unlink` round-trip. If e2e surfaces a real gap (e.g. inbound
   derivation not scanning backlog sources), it becomes red→green work inside this phase,
   with a regression test added — not deferred.

## Notes

- **Out of scope (design):** the review outlet F-5 (D2, deferred to Axis B), any
  `references`/role grammar (Axis B), coverage/close (Axis C), decomposition (Axis D),
  non-entity-target edges, and any consumer/graph-effect reaction (D3). No migration —
  this permits new edges, rewrites no stored row.
- **`governed_by` is not in VT-2's expected list** (no typed accessor emit), so only the
  `related` entry and the read_block golden change in the test surface; do not add a
  governed_by VT-2 entry.
- **Verification oracle is `inspect`**, not `backlog show` (label-selective summary).
