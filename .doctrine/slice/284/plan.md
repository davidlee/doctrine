# Implementation Plan SL-284: Embedded skills for non-Claude harnesses

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Reference forms: `lib:reference/glossary.md` § reference forms. -->

## Overview

Two independent strands, install-side then boot-side, then docs and the spec
Revision (REV):

```
PHASE-01  install: registry → .agents/skills; npx + skills_via + repo deleted;
          unknown --agent refused
PHASE-02  install: --only-memory subset (RV-423 F-1); gitignore hint (F-7)
PHASE-03  boot:    Harness::Pi; codex/pi arms split; MCP report sites (F-6)
PHASE-04  boot:    extensions → .doctrine/state/pi/extensions;
                   .pi/settings.json planner + outcome (F-4, F-8) + notice (F-5)
PHASE-05  docs, justfile, REV over SPEC-009/010/011
```

All serial: 01/02 share `src/install.rs`, 03/04 share `src/boot.rs`.

## Sequencing & Rationale

- **PHASE-01 first, whole.** Routing every harness through `skills` makes the
  delegation dead code in the same stroke; deleting it together keeps the tree
  free of a half-reachable path, and the compiler checks the removal (the
  `Leg::Skills` variant, the `Runner` seam). The unknown-agent refusal belongs
  here because it replaces the delegate's catch-all for unregistered names.
  It lands first to clear the shared surface for SL-283 (R3).
- **PHASE-02 separate.** `--only-memory` is a pre-existing bug and the hint is a
  block-rendering change; both are small, independent of routing, and would
  blur PHASE-01's test rewrite if folded in.
- **PHASE-03 before PHASE-04.** The adapter split is a pure ownership move with
  the extension path held constant, so its tests isolate "who writes" from
  "where". PHASE-04 then changes only "where" plus the new settings
  registration.
- **PHASE-05 last.** Docs describe the landed behaviour; the REV is raised once
  the behaviour it records is real, and applied at reconcile (design sec-6).

## Notes

- Design sec-3's test-table line numbers have drifted; locate the tests by
  their assertions (npx disclosure, `agents universal (delegate)`, offer text,
  codex writing pi files).
- Design sec-3's table omits `tests/e2e_install_ownership.rs`'s
  `pi_without_npx_*` tests, which assert removed behaviour; the table's own
  rule covers them (PHASE-01 EX-4).
- `write_codex_activation` is at `src/boot.rs:3099` (design cites `:3312`).
- VT keywords name tests this plan introduces; an executor that picks a better
  name updates the keyword in `plan.toml` in the same commit.
- Close-time items (not phase work): retire
  `mem.fact.doctrine.agents-skill-mirror-is-published-source`; backlog the
  `install_append_system` follow-up; VH-1/VH-2 in PHASE-05.
- Leave `skills-lock.json` and the foreign observation record untouched until
  the close-time cleanup.
