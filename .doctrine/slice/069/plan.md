# Implementation Plan SL-069: Shipped memory corpus as a cohesive client onboarding anchor

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Twenty-seven shipped memories: 13 new (Tiers 1–3 capability gaps) plus the 14
existing (refreshed for post-SL-018 staleness). The boot snapshot Memory section
shrinks from ~137 rows to ~16 signpost-only rows, and `boot --check` warns on
unpopulated governance sections. Four phases, file-disjoint where possible.

## Sequencing & Rationale

**PHASE-01 authors first, alone.** The 13 new memories touch only `memory/`.
They are the input to every other phase: PHASE-03 cross-references them,
PHASE-04 embeds and tests them. No dependency edge — PHASE-01 is the primary
producer, so it goes first. Authoring is the creative work; the other phases
are mechanical (filter, warn, refresh, integrate).

**PHASE-02 (boot changes) is file-disjoint from PHASE-01.** PHASE-02 touches
`src/boot.rs` and boot golden tests — zero overlap with `memory/`. This allows
PHASE-01 and PHASE-02 to run in parallel if desired. The design originally split
signpost filtering and governance warnings into separate phases, but they share
`src/boot.rs` and should be one atomic change: land both together or neither.
The signpost filter (`memory_type == "signpost"`) reduces 137 rows to ~16. The
governance warning adds a one-line markdown comment to empty Policies/Standards
sections and a `boot --check` warning row.

**PHASE-03 (corpus refresh) depends on PHASE-01.** It reads the full 27-memory
set (14 existing + 13 new from PHASE-01) against the current codebase. The
design provides a concrete handover checklist of 5 worst-stale memories
(§5.3) and a "do not rewrite" list of 8 substantively-correct ones. The phase
is designed for handover to a fresh agent — the consistency criteria and
cross-reference targets are well-defined. Cross-reference additions from new
to existing and vice versa bind the corpus into a connected graph.

**PHASE-04 (integration) is the serial final gate.** Every other phase must be
complete — the binary embeds the full 27-memory corpus, the boot snapshot
renders signpost-only, and governance warnings fire. Verification is holistic:
embed, sync, retrieval surface tests, and `just gate`. The 13 new integration
tests each assert both key match and ADR-002 shipped signature.

### File-disjointness

| Phase | Files touched | Parallel-safe with |
|-------|--------------|-------------------|
| PHASE-01 | `memory/` only | PHASE-02 |
| PHASE-02 | `src/boot.rs`, boot golden tests | PHASE-01 |
| PHASE-03 | `memory/` only | — (depends on PHASE-01) |
| PHASE-04 | integration tests, no new source | — (depends on all) |

### Risk: retrieval test sensitivity

Updating stale CLI verbs in `mem.signpost.doctrine.cli-command-map` or adding
cross-references changes shipped memory body text. The SL-005/007/008 retrieval
suites may have tests that assert on specific body content — those failures are
legitimate (the body changed, not the behaviour). The PHASE-03 agent must run
the gate before committing, distinguish legitimate body-change failures from
breakage, and update test oracles only when the body change is intentional and
correct (design §8).

### Risk: PHASE-03 scope creep

The 5 worst-stale memories need substantial rewrites (cli-command-map is ~80%
new content; file-map needs 6 new directory entries). The handover checklist
(design §5.3) constrains the agent to those 5 plus cross-reference additions
and minor wording fixes on the remaining 9. No new memories, no substantive
rewrites of substantively-correct memories.

## Notes

- The boot snapshot currently renders ~137 active memories (not just the 14
  shipped ones). The signpost-only filter is the architectural fix — design §3
  was corrected during review.
- No existing boot golden test asserts on specific memory rows. The signpost-only
  test in PHASE-02 is a new test, not an update to an existing one.
- Review (RV kind) shipped memory is deferred to a follow-up slice after SL-068
  lands — the review surface is in flux.
- OQ-1 (self-updating shipped memories) is flagged in the design but not solved
  in this slice.
- The ADR-002 signature (`repo=""`, `anchor_kind="none"`, scoped) is asserted
  in PHASE-04 integration tests for each new memory, not just key match.
