# Implementation Plan SL-114: Consolidate per-kind canonical_id onto shared listing helper

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

One phase, PHASE-01. The work is five mechanical, behaviour-preserving edits
across four files (`requirement`, `knowledge`, `backlog`, `spec`), all routing the
id-form `"{prefix}-{id:03}"` through the single authority `listing::canonical_id`
(`listing.rs:36`). There is no new behaviour, so nothing to build up in stages —
splitting it would add ceremony without isolating risk.

## Sequencing & Rationale

A single phase is correct here, not a default:

- **No staged dependency.** Each delegation is independent; none enables another.
  PHASE-01 in SL-113 existed because later phases built on a hardened primitive —
  there is no such primitive to land first here. The shared helper already exists.
- **One commit, one gate.** The whole change is verified by the same proof: the
  existing id-format tests (VT-1) stay green unchanged, the full suite stays green
  (VT-2), and the closure grep (VA-1) shows no surviving reimplementation. Spreading
  five edits across phases would re-run the same gate N times for no added signal.
- **TDD shape is inverted-but-honoured.** This is behaviour-preserving consolidation,
  so there is no red test to author — the *existing* tests are the red/green oracle.
  The refactor step is the whole phase. Closure gate EX-4 / VA-1 is the only net-new
  check, and it is an inspection (VA), not a test.

Spec's dual-wrapper collapse (D1) rides in the same phase: it touches `spec.rs`
alongside the spec method delegation, so doing it separately would mean two passes
over one file. Keep the method, delete the free fn, repoint its four in-file callers.

## Notes

- Implementation order within the phase is free; suggested: the three trivial
  delegations first (`requirement`, `knowledge`, `backlog`), then the `spec`
  collapse (method delegate + free-fn delete + 4 repoints) as the one multi-edit
  step, then run the closure grep and `just check`.
- Follow-up (NOT this slice, per scope §Non-Goals): a matching id-**parse** helper
  for the scattered `strip_prefix`/`id_from_fk` pattern — its own slice.
