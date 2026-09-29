# Implementation Plan SL-275: Memory search: default page, retrieval floor, and a zero-evidence signal

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Three phases, one per decision cluster, each ending green: paging (`DEC-347`),
then the floor and its signal (`DEC-348`–`DEC-350`), then shipped guidance.

## Sequencing & Rationale

**PHASE-01 first because it is independent and fixes live defects.** Paging
touches no ranking logic. It fixes the retrieve `--page` gap and the MCP overflow
(`RV-410` `F-2`/`F-4`), which exist today, and landing it alone keeps the
footer-arithmetic diff reviewable on its own. It also makes the e2e reachability
test page-independent before the floor changes which rows appear.

**PHASE-02 second because the floor changes row sets.** After PHASE-01, every
surface pages from one value, so a changed `total` under the floor shows up once
and consistently. The floor lives in the shared `query()` (`DEC-350`), so
this phase is also the behaviour-preservation gate for shared machinery: the
ranking, `sort_key` and surface-hook suites must stay green *unchanged*.

**PHASE-03 last because it describes shipped behaviour.** Guidance written before
the code lands would describe a contract that does not exist yet. It is a
docs-only phase, verified by keyword VTs plus an agent read against ADR-024's
shipped-corpus rules.

**Why not one phase.** The notice (PHASE-02) depends on both the resolved page
size and the floor, and the footer rewrite is the part the design review found
hardest to get right. Separating the two keeps each phase's failure surface small.

## Notes

- `format_truncation_notice` has a third caller (`src/priority/render.rs`). PHASE-01
  keeps aligned output byte-identical, so that caller's existing tests are the
  preservation check.
- The retrieve MCP path reuses the table renderer (`tools.rs` memory_retrieve).
  PHASE-02's notice must be checked there too (named as a residual probe in
  `notes.md`).
- **Path correction at plan time.** `design.md` (§Supersession and guidance, §Code
  impact) names `.agents/skills/...`; that tree is an untracked install
  projection. The shipped source is `plugins/doctrine/skills/...`, and the
  selectors were corrected to it. The locked design's intent is unchanged.
- MCP `memory_retrieve` renders table output, so it carries the no-match notice.
  The design's "no wire change" applies to `--json` and MCP `memory_search`.
- Paging tests need more than 20 memories. The existing `run_search` tests seed one
  memory through `temp_project_with_one_memory` (`run_record`). PHASE-01 EX-8
  generalises that helper instead of adding a parallel one; seeding 21+ memories
  through `run_record` is slower, but it is the real write path.
