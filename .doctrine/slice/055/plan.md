# Implementation Plan SL-055: Holistic skills review & token-efficient improvements

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three thin phases. This is a prose slice — few algorithms — so the plan stays
light per OQ-1. The weight is deliberately front-loaded: PHASE-01 buys a lean,
verified evidence base so the paired PHASE-02 spends Fable's scarce context on
judgement, not reading.

## Sequencing & Rationale

**PHASE-01 before PHASE-02** because the slice's whole token-economy rationale
(design §1-§3) is to have the expensive corpus reading already done — by external
cheap research (DeepSeek) and Opus synthesis — when Fable arrives. EX-3's
verification gate exists because the research wave is external and unverifiable
(design §9-A1): the high-severity staleness claims drive the most consequential
edits, so they are confirmed against ground truth before the paired session acts
on them, not after.

**PHASE-02 is the paired core.** It is ordered by the worklist's batches, not by
phase ceremony: the `close`-skill stale-debt batch first (highest value, all
verified prose, and it discharges CHR-004), then the lifecycle-hygiene reminders
that are this slice's primary reason for existing — the seam where each stage
skill should drive the ADR-009 transition and remind about its cross-artifact
obligations (close the originating backlog item; clean up merged worktrees). The
disposition model is the release valve: anything too structural for prose is
dispatched or backlogged rather than bloating the slice.

**PHASE-03 separates "edited" from "live and consistent."** Skill prose only
takes effect after the re-embed ritual, and prose edits can silently rot
cross-references; the integrity sweep is the cheap mechanical proof (design
§9-A3) that the corpus still hangs together before closure.

## Notes

The lifecycle this slice improves is also the lifecycle it runs under — the
status transitions were driven by the verb as we went (dogfooding BATCH 2). If
PHASE-02 surfaces a hygiene gap big enough to warrant a CLI affordance, that is
a `dispatch` or a backlog item, not a scope expansion of this slice.
