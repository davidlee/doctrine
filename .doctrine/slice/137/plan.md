# Implementation Plan SL-137: Corpus-level relation query verb — list edges by label, target, source-kind

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases, split on the pure/imperative seam the design mandates (§3, §5.1).
PHASE-01 is the whole pure engine — `relation_query.rs`, projections + renders
over `&Catalog`, fully unit-tested without disk. PHASE-02 is the thin impure
shell — clap surface, `scan_catalog`, the diagnostics policy, `RenderOpts`, and
the end-to-end golden. Every design VT lands in exactly one phase: the projection
VTs (VT-1..9, VT-12) in PHASE-01, the wiring/diagnostics VTs (VT-10, VT-11) in
PHASE-02.

## Sequencing & Rationale

**Why engine-first.** The design's centre of gravity is the pure projection
(§5.1): `&Catalog` in, rows/strings out, no clock/rng/git/disk. It is the part
that is cheaply and exhaustively testable over a seeded `Catalog` — so it is
built and proven first, against the design's interface contract (§5.2), before
any command exists to call it. This also keeps the layering proof trivial:
PHASE-01 has no command-tier dependency to introduce a cycle (ADR-001).

**Why the shell is its own phase.** The impurity the design isolates —
root-find, `scan_catalog`, stdout/stderr, terminal width/colour — is a distinct
responsibility (§5.4) and carries the two findings that survived the inquisition's
correction: the **diagnostics policy** (F1 — Error-only per-row, plus the bounded
edge-dropping summary line) and the **RenderOpts-in-shell** placement (F2). Both
are shell concerns; pinning them in PHASE-02 keeps the engine phase free of I/O
and lets VT-11 verify the corrected behaviour end-to-end. CLI registration (D1 —
`relation { list, census }`, link/unlink untouched) rides here too.

**Boundaries held, not re-opened.** The slice is pure consumption: no new
modelling, no write path, no transitive walk (SL-138), no export. The
validated-live-edge scope (design §5.5, inquisition F2) means illegal off-table
`[[relation]]` rows are out of scope by construction — `doctrine validate` owns
them; neither phase tries to surface them.

## Notes

- The `coverage_view` precedent (design §2) is the template for PHASE-01's
  bespoke-projection / shared-render shape — ride `listing`, do not fork it.
- Behaviour preservation (design §9): no edits to `catalog`/`relation`/`listing`
  internals in either phase; their suites stay green unchanged as the proof.
- Post-RV-139, the design is the contract. The only non-obvious build details are
  the F1 dropped-edge summary (count edge-dropping Warnings during the scan walk)
  and the F3 memory-target-by-UID matching — both already pinned in design §5.4.
