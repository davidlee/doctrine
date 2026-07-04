# Implementation Plan SL-201: Map focus on memory refs; onboarding command

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

One phase (`PHASE-01`). SL-201 closes two adjacent gaps — `--focus` rejecting
memory refs, and no human onboarding one-liner — over three file-adjacent
surfaces (`map.rs`, `cli.rs`, `guard.rs`). Design.md D1/D2 and § Reuse seam are
the canonical target.

## Sequencing & Rationale

**Why one phase, not two.** A tempting split is "focus support" then "onboard
verb". Rejected: the verb is a one-line delegation to `run_serve` with a
hard-coded focus, and it *depends* on the focus-resolution change to work at all
(its focus is a `mem.` key). Splitting would ship a phase whose only value is
consumed by the next, with no independent release point — phase overhead without
a boundary worth drawing. The two test clusters (VT-1/VT-2 focus, VT-3 verb)
live in the same inline `#[cfg(test)] mod tests` in `map.rs` and gate together.

**TDD order within the phase.** Red/green/refactor:
1. `validate_focus` — memory-ref branch. Tests VT-1 (accept key + uid), VT-2
   (reject malformed with a memory-ref message) first, then the `MemoryRef::parse`
   guard that makes them pass. Existing SL-001/numeric/bogus/empty tests stay
   green untouched (EX-3, behaviour-preservation).
2. `run_serve` — resolve a `mem.`-prefixed focus to its uid via
   `resolve_inspect_uid` before `Config`; unknown ref surfaces its `Err` ahead of
   the bind (EX-1).
3. `ONBOARDING_MEMORY_KEY` const + `run_onboard()` delegating to `run_serve`;
   VT-3 wiring assertion (EX-2).
4. `cli.rs` `Onboard` variant + dispatch arm; `guard.rs` classification.

**Verification split.** VT-1..3 are automated over the inline `map.rs` tests.
VH-1 is the browser round-trip (title-not-uid on the node, clear error on an
unknown key) — untestable in a headless gate, carried as a human check so it is
not silently skipped.

**Guardrails.** No `src/map_server/*` or `web/map/src/*` edits (design D2 — focus
already reaches the hash untouched). `doctrine check gate` clean before close
(EX-4).

## Notes
