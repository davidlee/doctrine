# Implementation Plan SL-054: Table cell wrapping for terminal-width-constrained output

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases that map one-to-one onto the design's three seams (design §3 D1,
§4 D2, §5 D3) — and, deliberately, onto the pure/impure boundary. Each phase ends
green; the dependency chain is strict (PHASE-01 enables PHASE-02 enables
PHASE-03), so the slice can run unattended without re-litigating earlier work.

The organising principle is **isolate the risk**. SL-054 carries three distinct
risks of different shape: a wide blast radius (the `RenderOpts` churn across 10
production callers), new branching logic (the Dynamic wrapping arm + grid floor),
and a purity-sensitive shell probe (crossterm `terminal::size`). Folding them into
one phase would make the behaviour-preservation gate un-bisectable. Splitting them
lets each phase carry exactly one kind of risk and prove it independently.

## Sequencing & Rationale

**PHASE-01 — the seam, behaviour-preserving.** The largest diff (10 prod sites +
4 test helpers + 2 wrappers + the `ListArgs` field) carries the *least* logic:
it is pure mechanical widening. By holding `render_table`'s body at `Disabled`
(the `Some(w)` arm deferred), every caller stays on the `None`/default path, so
the existing suite must pass **byte-for-byte unchanged** (VT-1). That unchanged
suite *is* the proof the blast radius landed clean — the cheapest possible gate on
the riskiest churn. Going first also means PHASE-02/03 build on a settled
signature, never re-touching the 10 sites.

**PHASE-02 — the engine, pure.** The actual wrapping lives here: the `Some(w)`
arrangement switch and the `grid_min_width` structural floor (the F-B fix from the
external review — a flat `MIN_WRAP_WIDTH` shreds wide tables, so the real fit test
must live where the grid is known: the pure layer). Because `render_table` is a
pure leaf, this phase is driven *entirely* by passing `Some(w)` directly in tests
— no terminal, no shell, no flake. The `force_no_tty`/edge-zero/`trim_end`
invariants stay unconditional (spike-proven orthogonal to wrap). At the end of
PHASE-02 the pure layer fully wraps when handed a width, yet real output is still
unchanged because the shell continues to hand it `None`.

**PHASE-03 — the shell, turn it live.** Only now does the impurity enter: the
`tty.rs` `terminal_width` pure split + the crossterm probe, wired into the three
resolution points (`into_list_args`, `coverage_view::run`, `priority::mod`). This
is the smallest phase but the one that flips behaviour for real TTYs. The
determinism gate (VT-2: goldens frozen) lands here because this is the first phase
where a width *could* leak into a pipe — so this is where we prove it cannot.

Why this order and not engine-first: the seam must exist before the engine has a
param to switch on, and the engine must wrap before the shell has anything worth
turning on. The chain is forced by the data flow, not by preference.

## Notes

- **Behaviour-preservation gate** (shared list-spine machinery): the existing
  in-crate + black-box suites are the proof and must stay green unchanged at every
  phase boundary (CLAUDE.md). PHASE-01 VT-1 and PHASE-02 VT-4 are the explicit
  None-invariance checks; PHASE-03 VT-2 is the piped-determinism check.
- **Exact caller inventory** is in design §3 (grep-derived, 10 prod sites incl.
  the easy-to-miss `rec.rs:576` empty-branch) — PHASE-01 EX-3 holds the
  completeness obligation the external review raised.
- **No new dependency** — crossterm rides in transitively via comfy-table's
  `custom_styling` feature; PHASE-03 VT-3 confirms with `cargo tree`.
- Manual `--width=N` / `--no-wrap` overrides are out of scope (design §9
  Follow-Ups); auto-detection only.
