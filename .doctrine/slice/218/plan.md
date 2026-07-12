# Implementation Plan SL-218: Tension narrative

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases tracking the design's own layering: semantics first (D7 knob),
then the pure data layer (detection + grading), then the surfaces (render).
Each phase lands independently green; the narrative only becomes visible at
PHASE-03, which is deliberate — the knob's trust semantics (design D1/D2)
must be honest *before* any stakeholder-shaped wording renders, per RFC-019
T7 and SL-217's pre-Phase-D obligation.

## Sequencing & Rationale

- **PHASE-01 first, alone.** It changes what "determined" means (knob-on)
  across every existing determinacy consumer — elicit queue, hypothetical
  outcomes. Shipping it against the untouched baseline gives a clean
  byte-identity proof (EX-2/VA-1: knob-off diffs are zero) before any new
  consumer exists, and PHASE-02's grades then inherit a single, already-
  verified system-selection seam rather than co-evolving with it.
- **PHASE-02 without render.** Detection and grading are pure data
  (EX-3 pins priority goldens unchanged), so the phase is fully unit-
  verifiable and reviewable without golden churn. The one-truth obligation
  (RV-271 F-1/F-7) is structural here — grades reuse the same
  `PairSide`/`determined()` calls the queue makes — and gets its
  observable e2e pin (VT-3 cross-surface agreement) at PHASE-03 when JSON
  makes grades visible.
- **PHASE-03 last: all golden churn in one reviewed diff.** Priority
  goldens change intentionally exactly once (design INV-1 rescope), with
  the four wording samples, cap, flag, and JSON additions pinned together.

Phases are sequential (each consumes the previous); no file-disjoint
parallelism is offered — PHASE-02/03 both touch `src/priority/`.

## Notes

- Verification ids map to the design's lettered VTs: design VT-A/B/C/F →
  PHASE-01 VA-1/VT-2/VT-3/VT-4; VT-D/G/J-state → PHASE-02 VT-1/VT-3/VT-2;
  VT-E/H/I/J-wording → PHASE-03 VT-1/VT-2/VT-3/VT-4.
- VT keyword mandates name identifiers the design commits to
  (`CompareConfig`, `demote_agent_evidence`, `TensionCause`,
  `AgentProposed`, `TENSION_MAX_CALLOUTS`, `human_only` for the subset
  compile entry, the disclosure and off-frontier literals). Workers
  satisfy the mandate by using the designed names — divergence from them
  is a design deviation, not a naming preference. Keywords were checked
  non-vacuous against the pre-phase tree (none match today).
- The `next` verbosity flag spelling is implementation-owned (design D5);
  the PHASE-03 VT-1 golden pins behaviour, not the flag name.
