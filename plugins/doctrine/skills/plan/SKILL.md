---
name: plan
description: Use when a slice's design is locked and it needs an executable phase plan — refine each phase's objective and entry/exit/verification criteria, author plan.toml + plan.md, and materialise the runtime phase sheets. Routed to from /design.
---

# Plan

You are turning design intent into an executable phase plan.

Inputs:

- `slice-nnn.md` (scope)
- `design.md` (canonical design reference)
- existing `plan.toml` / `plan.md` (when present)

## Process

1.  Read the slice scope and `design.md` together.
2.  Confirm planning is not getting ahead of design:
    - if `design.md` is missing or blank, stop and run `/design` unless you have
      been **explicitly instructed** otherwise.
    - if `design.md` exists but is stale relative to the current ask or slice
      scope, reconcile the design first. Clarify with the user if ambiguous.
    - you MUST NOT treat plan or phase creation as a substitute for unresolved
      design.
    - if planning surfaces substantive new design problems, run `/design` to
      revise or append to the design before continuing.
3.  Scaffold the plan: `doctrine slice plan <ID>` writes `plan.toml` + `plan.md`
    (refuses to clobber existing files). The tool reads these but never rewrites
    them — hand-edit freely.
4.  Author `plan.toml` — one `[[phase]]` per ordered phase:
    - `id` is `PHASE-NN` (zero-padded), **immutable** and never reused — edits
      append, they never renumber.
    - `name` and `objective` for the phase.
    - `entrance_criteria` (`EN-n`), `exit_criteria` (`EX-n`), and `verification`.
      Verification ids carry their mode: `VT-n` verified by test, `VA-n` by
      agent, `VH-n` by human — use `VA`/`VH` when a test cannot judge the
      criterion, so it is still checked downstream rather than silently skipped.
      These ids are local to the phase and equally immutable.
    - `specs` / `requirements` stay empty in v1 (no registry yet). When a slice
      does carry relations, they are written with `doctrine link` (not typed keys
      here) — see `using-doctrine.md` § Relating entities.
5.  Author `plan.md` — the rationale and sequencing prose: why these phases, in
    this order, with these boundaries. Honour the storage rule: **no queried or
    derived data in `plan.md`**; the structured criteria live in `plan.toml`, and
    runtime progress lives under `.doctrine/state/`, never here.
6.  Materialise phase tracking: `doctrine slice phases <ID>` creates the per-phase
    sheets in the state tree from `plan.toml`. `--prune` removes orphan tracking
    whose plan phase is gone (destructive — only when you meant to drop a phase).
7.  Consider the finished plan critically: what's under-specified, assumed, ambiguous,
    oversized, optimistic, requires verification, or presents implementation risk?
    For each of these, decide what new information is needed, or what mitigations
    to put in place. Ground the plan in known details of the implementation.
8.  Plan the revision; print a summary of it for the user. Then work through it.
9.  Revise the plan; summarise what changed.
10. Hand off to `/phase-plan` to expand the next phase's runtime phase sheet
    just before execution — then `/execute`. Do this only after slice scope, `design.md`, and
    the plan tell the same story. Record the lifecycle move on handoff:
   `doctrine slice status <id> ready` (bare number).

## Outcomes

- `plan.toml` is execution-ready: every phase has an objective and EN/EX
  criteria plus verification in an explicit mode (VT/VA/VH).
- `plan.md` explains the rationale and sequencing.
- A runtime phase sheet exists per phase with clear done criteria.
- If plan complexity or policy ambiguity emerges, STOP and `/consult`.
