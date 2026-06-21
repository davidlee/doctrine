# Implementation Plan SL-139: Uniform entity show and file path surfaces

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases, sequenced by dependency: the shared helper first, then the isolated
show-parity fix, then the paths verb rolled out across two batches (numeric stem
kinds first as the common pattern, then umbrella and named kinds as the variant
adapters). Each phase is independently testable and produces a green `just gate`.

## Sequencing & Rationale

### PHASE-01 → PHASE-02, PHASE-03

`src/paths.rs` is the foundation. Every subsequent phase calls into it. It must
exist and pass its tests before any CLI wiring begins. PHASE-02 (concept-map
`--json` shorthand) is independent of the paths helper but listed second because
it is small, self-contained, and a quick confidence-building win before the
larger rollout.

### PHASE-03 → PHASE-04

Numeric stem kinds (adr, policy, standard, rfc, slice, review, rec, revision,
concept-map) share a common adapter pattern: parse a numeric ref, construct
`{stem}-NNN.{toml,md}` paths under a flat directory. Governance kinds (adr,
policy, standard, rfc) already share a dispatch spine via `governance.rs` and
will share a single code path — minimizing duplication.

PHASE-04 (backlog, spec, knowledge, memory) requires prefix-to-sub-directory
dispatch which is a variant on the pattern proven in PHASE-03. Delaying this
batch avoids blocking the common case on the harder variants.

### Why not merge PHASE-03 and PHASE-04?

Nine numeric kinds in one phase is already a large surface. Adding four
prefix/named kinds would bloat the phase and make it harder to isolate failures.
The two batches are file-disjoint (different command modules) and could be
parallelized if desired.

### Why is concept-map --json its own phase?

It is zero-dependency on the paths helper, tests an entirely different code path
(clap flag dispatch, not filesystem scanning), and is trivially small. Keeping it
separate avoids coupling the clap parsing of 13 commands with a standalone
cosmetic fix.

## Notes

- All phases are additive — no existing behaviour is removed or restructured.
  Existing test suites must stay green after each phase.
- `just check` (fast inner loop, root crate only) should pass after each phase.
  `just gate` (full workspace, clippy zero-warnings) is the pre-commit gate.
- The SPEC-013 reconciliation (adding `paths` to the uniform verb set) is
  scheduled for the reconcile phase per design §7 D7 — not during implementation.
- ADR-001 layering enforcement runs automatically via `tests/architecture_layering.rs`
  under `just gate`. The new `src/paths.rs` module will need a layering.toml entry.
