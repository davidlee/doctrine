# IDE-053: New stub agent role + rewrite orchestrator/worker role prompts for the clone workflow

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

`SL-254` PHASE-10 deleted the shipped `install/hymns/role/orchestrator.md`
whole — it was Mode B written as a session-start role prompt, resolved into
**every** doctrine client's context at session start via `doctrine prompt
resolve --role orchestrator` (`src/boot.rs`'s `SessionStart` hook wiring), not
a dispatch-only artifact. The one arm-agnostic fact it carried (the
report-and-halt boundary on trunk-facing ops) already lives verbatim in
`install/dispatch-mechanics.md`, so nothing true was lost by deleting it — but
the shipped default for the `orchestrator` role band is now empty for any
project that hasn't authored its own override (this repo has:
`.doctrine/hymns/role/orchestrator.md`, an arm-agnostic "you own the process,
the plan, and the decision" stub).

## The idea

Owner direction (2026-08-13), explicitly flagged as **possible scope creep
needing its own discussion — not decided, not authorised for immediate
implementation**:

1. A new session-start default role, **`agent`** — mostly an empty stub for
   now, presumably the generic/no-specific-posture default every
   doctrine-installed project's clients get absent a project override.
2. Rewrite the `orchestrator` role prompt for **the git-clone workflow** —
   language suggesting this is downstream of `SL-255` (clone provisioning),
   once workers/orchestrators have a concrete clone-based shape to describe.
3. Rewrite `install/hymns/role/worker.md` similarly — `SL-254` PHASE-10 only
   did the minimal edit (drop the dead Mode-B clause, keep the surviving
   subprocess-worker description); this idea's fuller rewrite is a separate,
   later pass once the clone workflow exists to describe.

## Why this wasn't done in SL-254 PHASE-10

`SL-254`'s objective is the arm collapse, not a role-taxonomy redesign. A new
`agent` role and clone-workflow-specific role prompts both presuppose design
decisions `SL-254` doesn't own (what does "clone workflow" mean operationally,
who is `agent` for and what differentiates it from `orchestrator`/`worker`).
PHASE-10 did the minimum honest thing — delete/edit what was actively wrong —
without pre-empting this larger design.

## Open questions for whoever picks this up

- Does `agent` supersede `orchestrator`/`worker` as role names, or sit
  alongside them as a third, more general default?
- Does this wait on `SL-255` landing (so "the clone workflow" has a concrete
  shape to write role prompts against), or can the `agent` stub ship
  independently first?
- Relationship to `IDE-052` (reviving `/drive-slice` for main-worktree
  sequential clone drive) — if that revival happens, its driver pattern may be
  exactly what an `orchestrator` role prompt for the clone workflow should
  describe.
