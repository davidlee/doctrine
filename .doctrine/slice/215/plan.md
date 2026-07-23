# Implementation Plan SL-215: Unified harvest surface

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five small phases over a prose-only touch-set (13 skill/doc targets + the notes
template; no engine/CLI change). The design (`design.md`) is canon: one owner
doc (`install/harvest.md`), one canonical output (the maintained `## Harvest`
section), `/notes` as the routed entry, every other tail a citation, and the
IMP-042 cadence leg on the code-review skill.

## Sequencing & Rationale

- **PHASE-01 first, honouring the IMP-042 → IMP-059 `after` edge.** The cadence
  section is independent of the harvest doc, so it lands before the harvest
  machinery exists. It also keeps the code-review skill's body stable before
  PHASE-04 shrinks its tail — two edits to one file, but on disjoint sections,
  in a deliberate order.
- **PHASE-02 before any consumer edit.** Nothing may cite `harvest.md` before
  it exists; the template stub and the ledger §5 defer land with it because all
  three are seams of the same owner-doc move.
- **PHASE-03 (entry + projections) before PHASE-04 (shrinks).** The projections
  define what "consume, don't re-survey" means in skill text; the shrinks then
  copy that established citation form across the remaining six tails. Reversing
  the order would have the shrinks invent the form the projections later
  contradict.
- **PHASE-05 is the single delivery gate.** Re-embed → build → fresh-client
  install → POL-002 resolve → corpus grep → dogfood. One gate at the end rather
  than per-phase installs: the re-embed footgun (`touch` the embedding crate or
  the new file is invisible) makes per-phase install churn expensive and
  valueless — nothing consumes the installed copies mid-slice.

Behaviour preservation: the design's VA scope holds the four named projections
(`/notes`, `/handover`, `/next`, review synthesis) to their existing contracts;
the six shrink targets change only their harvest tails, asserted by keyword
mandates that pin the surviving lens text (e.g. audit's phase-sheet sweep,
close's "consciously rejected").

## Notes

- VT mandates on markdown targets are substring floors (POL-002: raw source,
  no stripping) — they gate presence of the load-bearing strings, not prose
  quality; VA rows carry the semantic checks.
- The dogfood mandate (PHASE-05 VT-1) targets `.doctrine/slice/215/notes.md`,
  which does not exist until the harvest runs — `test_file` must exist only
  after the phase lands, which it does.
- Residuals consciously carried from design §6: the adherence bar stays a
  qualitative heuristic; harvest staleness is honour-model. Neither is a plan
  gap.
