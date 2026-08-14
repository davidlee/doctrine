# REQ-290: The orchestrator runs on a dedicated, markerless dispatch/<slice> coordination worktree provisioned per run, created inside the project root and held across handover-resume with Bash cwd parked there; concurrent same-slice dispatch is refused at creation and the worktree is removed at conclude while the branch is kept.

## Statement

> **AMENDED — VOCABULARY ONLY (SL-254, 2026-08-14).** Every clause of the title
> line still holds. The word **"markerless"** is now vacuous rather than
> distinguishing: the disk marker it contrasted against is gone (`DEC-207`), so
> every tree is markerless and the qualifier no longer separates the coordination
> worktree from anything. Read it as: the coordination worktree is NOT a worker —
> the orchestrator process does not carry `DOCTRINE_WORKER`, so `worker_guard`
> does not refuse its Write- and Orchestrator-classed verbs. The title is
> retained unchanged; no substantive requirement changes.

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
