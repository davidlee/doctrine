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

1. Read the slice scope and `design.md` together.
2. Confirm planning is not getting ahead of design:
   - if `design.md` is missing or blank, stop and run `/design` unless you have
     been **explicitly instructed** otherwise.
   - if `design.md` exists but is stale relative to the current ask or slice
     scope, reconcile the design first. Clarify with the user if ambiguous.
   - you MUST NOT treat plan or phase creation as a substitute for unresolved
     design.
   - if planning surfaces substantive new design problems, run `/design` to
     revise or append to the design before continuing.
2a. **Requirements mapping.** Read `design-requirements.toml` — collect every
    `REQ-DNN` handle. For each, determine which phase(s) will verify it. Read
    any canonical REQs cited in the design. Present the mapping to the user
    for confirmation.

    Record the mapping in `plan.md` under a `## Requirements verification`
    section as **narrative prose with a list** (not a pipe-table — a table
    implies machine-parseable structure this design explicitly disclaims):

    ```markdown
    ## Requirements verification

    This slice verifies the following implied requirements (handles from
    `design-requirements.toml`) and cited canonical REQs:

    - REQ-D01 (audit trail retention) — verified by PHASE-01 and PHASE-02
      (write-seam + record format).
    - REQ-D02 (orphan detection gate) — verified by PHASE-03 (close gate
      implementation).
    - REQ-077 (canonical, cited by design) — verified by PHASE-01.
    ```

    This section is authored, agent-read narrative — not tool-queried, not
    derived. The close gate reads it as the LLM reads any prose to confirm
    every `REQ-DNN` has a `→ REQ-NNN` mapping. This mapping lives in prose
    because (a) `plan.toml [requirements]` is the v1-empty registry, not a
    verification map, and (b) `design-requirements.toml` is owned by `/design`
    and exists before `/plan` runs.

    **Guardrail:** If a `REQ-DNN` has no phase assigned, that's a design gap —
    surface as a `/consult` trigger. Do not drop the requirement silently.

    **Advisory, not enforced.** The close check is agent discipline, not a
    binary gate: nothing in the CLI refuses close on an unplaced orphan. The
    close skill's walkthrough is the backstop.
3. Scaffold the plan: `doctrine slice plan <ID>` writes `plan.toml` + `plan.md`
   (refuses to clobber existing files). The tool reads these but never rewrites
   them — hand-edit freely.
4. Author `plan.toml` — one `[[phase]]` per ordered phase:
   - `id` is `PHASE-NN` (zero-padded), **immutable** and never reused — edits
     append, they never renumber.
   - `name` and `objective` for the phase.
   - `entrance_criteria` (`EN-n`), `exit_criteria` (`EX-n`), and `verification`.
     Verification ids carry their mode: `VT-n` verified by test, `VA-n` by
     agent, `VH-n` by human — use `VA`/`VH` when a test cannot judge the
     criterion, so it is still checked downstream rather than silently skipped.
     These ids are local to the phase and equally immutable.
   - `specs` / `requirements` stay empty in v1 (no registry yet). The `REQ-D →
     phase` verification mapping lives in `plan.md` prose (sub-step 2a), not
     here. This is a trade — when the requirement registry lands, the mapping
     graduates to `plan.toml`. When a slice does carry relations, they are
     written with `doctrine link` (not typed keys here) — see
     `using-doctrine.md` § Relating entities.
5. Author `plan.md` — the rationale and sequencing prose: why these phases, in
   this order, with these boundaries. Honour the storage rule: **no queried or
   derived data in `plan.md`**; the structured criteria live in `plan.toml`, and
   runtime progress lives under `.doctrine/state/`, never here. Authored,
   agent-read verification narrative (the `## Requirements verification`
   section — sub-step 2a) is a permitted exception: it is authored by the
   planner, read by the agent, and distinct from tool-queried or derived data.
6. Materialise phase tracking: `doctrine slice phases <ID>` creates the per-phase
   sheets in the state tree from `plan.toml`. `--prune` removes orphan tracking
   whose plan phase is gone (destructive — only when you meant to drop a phase).
7. If plan complexity or policy ambiguity emerges, `/consult`.
8. Hand off to `/phase-plan` to expand the next phase's runtime phase sheet just before
   execution — then `/execute`. Do this only after slice scope, `design.md`, and
   the plan tell the same story. Record the lifecycle move on handoff:
   `doctrine slice status <id> ready` (bare number).

## Outcomes

- `plan.toml` is execution-ready: every phase has an objective and EN/EX
  criteria plus verification in an explicit mode (VT/VA/VH).
- `plan.md` explains the rationale and sequencing.
- A runtime phase sheet exists per phase with clear done criteria.
