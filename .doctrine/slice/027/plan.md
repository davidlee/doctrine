# Implementation Plan SL-027: DRY backlog test-fixture TOML builders into one helper

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

A single-phase, test-only refactor. The debt (ISS-001) is one TOML literal
copied across four sites in the `src/backlog.rs` test module; the design
(`design.md` D1) lifts it into one core builder and lets the existing named
helpers delegate. There is no production surface and no behaviour change, so the
backlog suite — passing unchanged — is the whole proof.

## Sequencing & Rationale

**Why one phase.** The change is mechanical extract-and-delegate under a
byte-equivalence invariant already verified line-by-line in `design.md` §4. There
is no internal ordering risk to split on: the core builder, the three wrappers,
and the `:1813` fold all land together or the suite goes red. Splitting would
manufacture artificial intermediate states (a half-migrated helper set) with no
review value.

**Why TDD here is "keep green," not "red→green."** This is a refactor, not new
behaviour. The red/green/**refactor** loop collapses to its refactor leg: the
existing assertions are the red-bar proxy, held green throughout. The discipline
is to change *only* helper bodies and the one `:1813` call site, never an
assertion — any assertion edit would mean the output drifted, defeating the
behaviour-preservation gate.

**The scope boundary is load-bearing.** Three inline literals (`:1161`, `:1190`,
`:2075`) feed bytes straight to the parser / error path and must keep showing
their exact bytes; they are fixtures-under-test, not fixture *builders*, and
EX-4 forbids touching them. The closure metric (VA-1) lands at 4, not 1,
precisely because of them.

## Notes

- API shape settled in `design.md` D1 (extract-core-keep-wrappers; the
  collapse-to-one-builder alternative was rejected for ~30-site churn).
- `Fixture` carries a lifetime `'a` (not `'static`) because `write_related`
  passes borrowed `slices`/`specs`.
- `render_fixture_toml` concatenates `format!` segments rather than
  `push_str(&format!(..))`, honouring the repo string-build convention even
  though the gate's `cargo clippy` does not lint `cfg(test)` code.
