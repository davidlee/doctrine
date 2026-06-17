# SL-085 — implementation notes

## Design decisions (durable)

- **D1 (dropped `dispatch import`):** The verb saves zero tokens — `worktree import` is already a one-liner. An alias adds CLI surface without reducing skill prose. Unanimous with user.
- **D4 (blocked-phase exclusion):** `plan-next` skips blocked phases in `next` output. Blocked = not actionable for dispatch. All-remaining-blocked prints explicit `(none)` message. User confirmed: surface the block (visible in rollout) but exclude from next (prevent costly error of dispatching blocked work).
- **D5 (no file-disjointness v1):** Adding a `files` field to plan.toml is an authored schema change out of scope. The orchestrator still runs `/phase-plan` for file sets. Deferred; revisit when plan.toml schema evolves.
- **D6 (trunk drift in status):** `git merge-base --is-ancestor <fork_point> <live_trunk>` catches divergence early, before integrate-time CAS refusal. Fork-point recomputed each invocation — no stored state.

## Review findings (all resolved)

7 adversarial self-review findings, all dispositioned in design.md. No unresolved. No governance conflicts. See handover.md for the table.

## Audit findings (RV-067, 2026-06-18)

4 findings raised, all terminal (verified). No blockers.

- **F-1 (major, tolerated):** `dispatch/085` coordination branch absent — likely GC'd after conclude. Reviewed `review/085` directly. Process gap: dispatch ref should survive until after audit.
- **F-2 (minor, aligned):** Skill body line counts at design ceilings (45/45, 29/30, 24/25). Within targets, content correct. Zero headroom for future additions.
- **F-3 (minor, tolerated):** ISS-019 (plan.toml-not-found on provisioned worktrees) carried forward unchanged by mechanical extraction.
- **F-4 (minor, tolerated):** IMP-091 (corrupt patch in `worktree import`) — worker imports used checkout fallback. Functionally equivalent, but workers lose automatic GC.

### Standing risks

1. **Line-count headroom zero:** The dispatch router skill is exactly at the 45-line body limit. Any addition will push it over.
2. **Dispatch ref lifecycle:** No mechanism prevents premature GC of the coordination branch before audit.
3. **IMP-091 / ISS-019:** Pre-existing bugs carried forward — both have backlog items tracking root causes.
