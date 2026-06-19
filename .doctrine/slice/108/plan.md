# Implementation Plan SL-108: pi dispatch worker integration via RPC mode

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases. PHASE-01 is the authored deliverable — one skill file update, one
test cap bump. PHASE-02 is the validation gate — an end-to-end exercise proving
the pi RPC worker integrates cleanly with the dispatch funnel.

## Sequencing & Rationale

PHASE-01 must complete before PHASE-02 because the e2e exercise uses the
documented spawn template from the updated skill. The phases are strictly
sequential.

PHASE-01 is small and mechanical: copy the design.md spawn template into the
skill file, preserve the existing codex arm, bump the shrinkage cap. No design
decisions remain — the design.md is the spec.

PHASE-02 exercises the whole cadence: fork, marker provision, spawn, wait for
agent_end, extract outcome, import, commit. It also validates the extraction
fallback ladder against synthetic payloads (per D1) and timeout enforcement
(per D1 timeout spec). This phase exercises the integration, not just the
template syntax. The extraction ladder verification is manual — construct a
synthetic `agent_end` JSON payload with (a) no assistant messages and (b) a
tool-call-only assistant message, and confirm the status derivation produces
`no_output` and `partial` respectively.

## Notes

- No CLI verb changes, no binary changes — this slice is documentation + e2e
  validation only.
- The `[dispatch] preferred_subprocess_harness` config key is deferred to
  IMP-101; this slice documents the expected key in the skill but does not
  implement config reading.
- Raw `agent_end` JSONL token pressure is deferred to a post-hoc digest.
