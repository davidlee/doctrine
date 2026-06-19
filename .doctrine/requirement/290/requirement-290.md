# REQ-290: The orchestrator runs on a dedicated, markerless dispatch/<slice> coordination worktree provisioned per run, created inside the project root and held across handover-resume with Bash cwd parked there; concurrent same-slice dispatch is refused at creation and the worktree is removed at conclude while the branch is kept.

## Statement

<!-- The requirement in full: what must hold, stated testably. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
