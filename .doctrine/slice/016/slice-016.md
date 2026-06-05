# Break slice↔state cycle: extract plan types

## Context

Modularity/coupling assessment of `src/` (2026-06-05, ~15kLoC flat tree, 16
files). Verdict: structurally honest — clean layering, `entity` is a proper
cohesive core hub, leaf utilities (`clock`/`root`/`fsutil`) and command leaves
(`boot`/`adr`/`skills`) sit where they should. ~51% of LoC is inline tests
(prod ~7.2k). For the size, a B+. No structural rot.

One genuine defect surfaced: a **`slice ↔ state` import cycle**.

```
slice -> state   (init_phases, set_phase_status, PhaseRollup — real calls)
state -> slice   (use crate::slice::{Plan, PlanPhase} — types only, one line)
```

Cause is purely **data-type placement**: the authored phase-plan model
(`Plan`, `PlanPhase`, `impl Plan::parse`) lives in `slice.rs` (slice-014 era),
but `state.rs` needs those types to materialise phase tracking sheets. Not a
behavioural entanglement — a misplaced struct.

The layering rule this slice enforces is recorded project-wide in **ADR-001**
(module layering: leaf ← engine ← command, no cycles). SL-016 is its first
application.

A false alarm was ruled out in the same pass: `git -> retrieve` is a
doc-comment link, not code. `git` is a clean leaf seam. The only real cycle is
this one.

The name `Plan` is also **overloaded** across three unrelated modules
(`slice.rs` authored-plan, `install.rs` installer-plan, `skills.rs`
skills-install-plan). Only the slice variant is in scope here; the overload is
noted as a naming smell, not addressed.

## Scope & Objectives

Relocate the authored phase-plan model out of `slice.rs` into its own module so
the dependency graph becomes acyclic.

- Extract `Plan`, `PlanPhase`, and their parse logic (`impl Plan`, the `Raw`
  deserialisation shim) into a new `plan` module.
- Repoint consumers: `state` and `slice` both import from `plan`. Resulting
  edges — `slice → plan`, `state → plan`, `slice → state`. Cycle gone.
- Behaviour-preserving: this is a pure move. Every existing suite stays green
  **unchanged** (the behaviour-preservation gate). No new behaviour, no API
  shape change beyond the module path.
- Decide where disk-reading `read_plan` lands (pure `Plan::parse` belongs in
  `plan`; the disk read may stay in `slice` or move) — a `/design` question.

## Non-Goals

- The `Plan` name overload across `install`/`skills` — separate naming concern,
  not touched here.
- Folder-grouping the flat `src/` tree (`engine/`, `command/`, `seam/`) — latent
  in the graph, borderline-not-painful at 16 files. Deferred; see Follow-Ups.
- Splitting `memory.rs` (1193 prod LoC, largest module) — watch item, not yet a
  defect. Deferred.
- Any change to phase-plan semantics, TOML schema, or CLI surface.

## Summary

Move ~30 lines of type + parse code from `slice.rs` to a new `plan.rs`, fix the
imports in `state.rs` (and the constructor site in `main.rs`). Smallest change
that turns the module graph into a clean DAG. Verified by an unchanged green
test suite plus a confirmation that no module imports `slice` for plan types.

## Follow-Ups

- **Flat-tree folder grouping** — defer until file count (~20+) or onboarding
  friction forces it. Natural cut: `engine/` (entity, state, meta, fsutil,
  clock, root, input), `command/` (slice, adr, memory, retrieve, boot, skills,
  install), `seam/` (git).
- **`memory.rs` god-module watch** — 1193 prod LoC; split store/query if it
  keeps growing.
- **`Plan` name overload** — three unrelated `Plan` types; consider
  disambiguating names (`InstallPlan`, `SkillsPlan`, `PhasePlan`).
