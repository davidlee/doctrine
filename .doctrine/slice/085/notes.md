# SL-085 — implementation notes

## Design decisions (durable)

- **D1 (dropped `dispatch import`):** The verb saves zero tokens — `worktree import` is already a one-liner. An alias adds CLI surface without reducing skill prose. Unanimous with user.
- **D4 (blocked-phase exclusion):** `plan-next` skips blocked phases in `next` output. Blocked = not actionable for dispatch. All-remaining-blocked prints explicit `(none)` message. User confirmed: surface the block (visible in rollout) but exclude from next (prevent costly error of dispatching blocked work).
- **D5 (no file-disjointness v1):** Adding a `files` field to plan.toml is an authored schema change out of scope. The orchestrator still runs `/phase-plan` for file sets. Deferred; revisit when plan.toml schema evolves.
- **D6 (trunk drift in status):** `git merge-base --is-ancestor <fork_point> <live_trunk>` catches divergence early, before integrate-time CAS refusal. Fork-point recomputed each invocation — no stored state.

## Review findings (all resolved)

7 adversarial self-review findings, all dispositioned in design.md. No unresolved. No governance conflicts. See handover.md for the table.
