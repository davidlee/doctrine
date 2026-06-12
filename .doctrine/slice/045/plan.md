# Implementation Plan SL-045: Requirement status visibility: spec req roster + standalone drift read

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two read-only surfaces over shipped seams (design §5.1) — no new engine, no new
store. The plan threads them as **leaf-up**: the two underlying seams first
(batched scanner, spec fan), then the pure view that joins them, then the
authored-only roster, then CLI wiring, then the cross-cutting seam proof. Each
phase ends green and additive; the behaviour-preservation gate (F6) holds at
every boundary.

## Sequencing & Rationale

The phase boundaries follow the design's module homes, ordered by the
dependency DAG rather than by command:

- **PHASE-01 (batched scanner) first** because it is the shell seam the derived
  read stands on, and it is the RSK-006 fix (D4) — a spec fan must be one corpus
  walk, not N. It is independent of every other phase and carries the heaviest
  behaviour-preservation risk (it touches the sole git/disk seam), so it goes
  first and proves itself against the existing suite before anything rides it.
  Q1 (fold vs keep both) is decided here, empirically, by byte-identical
  behaviour — not pre-committed.

- **PHASE-02 (spec fan seam) second**, independent of PHASE-01. It is small but
  load-bearing: the canonicalization fix (E2) is the difference between a read
  that reports truth and one that silently fabricates non-divergence. Isolating
  it as its own phase makes the silent-false-negative guard the phase's reason
  to exist, not a footnote in a larger change.

- **PHASE-03 (coverage view leaf)** depends on both seams (EN-1). This is where
  the two blockers converge structurally — the `CoverageRow` enum (E1) and the
  `observed_state` partition (E6) — and where the F1 wall is first expressed in
  code (authored status joined display-only, never derived). Pure compute, so it
  is unit-testable without the CLI.

- **PHASE-04 (spec req list roster)** is the authored-only surface. It rides the
  spec membership read directly and is deliberately walled off from the derived
  tier (D3, INV-3): no scan, no observed/verdict column. Sequenced after the
  view leaf only for reviewer cohesion (both reads exist before wiring); it has
  no hard code dependency on PHASE-03. The E5 symmetric-degrade fix lives here.

- **PHASE-05 (CLI wiring)** waits for both reads (EN-1). Kept distinct from the
  leaves so the leaves stay unit-tested and pure; this phase only dispatches and
  threads `--columns`/`--json`, ending at a smoke-green, clippy-clean binary.

- **PHASE-06 (seam wall + goldens) last** because INV-1 must be pinned at the
  **command seam** (A3), not a pure helper — it can only exist once the command
  renders. The black-box goldens and the F6 confirmation are also seam-level, so
  they batch here as the slice's closing proof.

Why not merge wiring (05) into the goldens phase (06): the wiring is a small,
mechanical dispatch step whose own success is a smoke test; the seam proof is a
substantial, behaviour-defining test layer (the wall, every golden surface).
Separating them keeps "make it runnable" honest from "prove it correct."

## Notes

- specs/requirements stay empty in `plan.toml` (v1 — no requirement registry yet);
  lineage is recorded in the design (descends SPEC-002 / PRD-013; realises the
  user-facing half of REQ-110 / REQ-111).
- The design is authority — this plan does not re-litigate it. All scope OQs and
  both adversarial passes are resolved in design.md §6/§7/§10.
