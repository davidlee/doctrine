# Review RV-067 — reconciliation of SL-085

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed:** `refs/heads/review/085` (tip: `4950b40dbb58`). Candidate workflow
unavailable — `dispatch/085` ref is absent (likely GC'd after conclude by a
coordinating agent; review/085 was prepared externally). Auditing the impl-bundle
directly, noting the missing ref as a non-blocking process finding.

**Lines of attack:**

1. **Design conformance (D1–D10).** Are all 10 design decisions satisfied in the
   implementation? Focus: D1 (no dispatch import in skills), D2 (plan gate on setup),
   D3 (env contract), D4 (blocked-phase exclusion), D6 (trunk drift), D8 (shared
   renderer), D9 (coordinate extraction preserves behaviour).

2. **Verification conformance (plan.toml VT criteria).** Do all 18 VT + 1 VA criteria
   hold? Running `just check` confirms gate-green for code phases; PHASE-04 shrinkage
   and embedding verification needs source inspection.

3. **Behaviour-preservation gate.** Does the `worktree::coordinate()` extraction leave
   `run_coordinate` unchanged? All existing worktree coordinate integration tests must
   stay green.

4. **Skill shrinkage accuracy.** The three dispatch skills must reference the new
   verbs (`dispatch setup`, `dispatch plan-next`, `dispatch status`) correctly, purge
   all `dispatch import` references (D1), and stay within target line counts.

5. **Watch-out follow-through.** Two pre-existing bugs acknowledged: IMP-091
   (corrupt-patch in `worktree import`, checkout fallback used) and ISS-019
   (plan.toml-not-found on provisioned worktrees).

## Synthesis

SL-085 pushes the dispatch drive loop from ~909 lines of skill prose into three
new CLI verbs (`dispatch setup`, `dispatch plan-next`, `dispatch status`) plus a
mechanical extraction of `worktree::coordinate()`. The result is a net reduction
of ~500 lines from the skill files (321→49, 162→33, 119→28 body lines within
targets) with ~1,364 lines of new Rust across `src/dispatch.rs`, `src/worktree.rs`,
`src/main.rs`, and `src/state.rs`.

### Design conformance: green

All 10 design decisions (D1–D10) are satisfied. The implementation follows the
design faithfully:

- **D1:** No `dispatch import` references survive in any skill — the verb was
dropped from scope, and the integration test asserts its absence.
- **D2–D3:** `dispatch setup` gates on plan.toml existence + non-empty phases
before touching git, then emits the `KEY=value` env contract on stdout (human
status on stderr).
- **D4:** `plan-next` skips blocked phases in `next` output. In-progress phases
gate subsequent pending. All-blocked prints an explicit `(none)` message.
- **D6:** Trunk drift is stateless — recomputed each invocation via
`git merge-base` + `git rev-list --count` against the live trunk tip.
- **D8:** `render_phase_table` is a shared helper called by both `plan-next` and
`status`.
- **D9:** `worktree::coordinate()` is extracted as a pure-ish core returning
`CoordOutcome { dispatch_tip }`. `run_coordinate` wraps it with CLI I/O — existing
integration tests stay green (behaviour-preservation gate).
- **D10:** `plan-next` and `status` are Read-classed; `setup` is Orchestrator.

### Verification conformance: green

All 18 VT criteria and 1 VA criterion from plan.toml are satisfied: `just check`
passes (1642 tests, 0 failures, clippy zero warnings). The 6 existing worktree
coordinate tests remain green. The skill embedding verification test
(`e2e_skills_dispatch_shrinkage.rs`) asserts body line counts ≤target, key prose
presence, and `dispatch import` absence — all passing.

### Standing risks

1. **Line-count headroom is zero.** The router skill (dispatch/SKILL.md) is at
exactly the 45-line body limit. Any future addition — a new red flag, a new
guidance line — will push it over. The arm skills have marginally more room
(1–2 lines each).

2. **Dispatch ref lifecycle gap.** The `dispatch/085` coordination branch was
removed before audit, making the candidate workflow unavailable. The ref should
survive until after audit/reconciliation per the design, but the current
dispatch conclude path (remove coord worktree → branch is GC'd) does not enforce
this.

3. **Two pre-existing bugs carried forward unchanged:** IMP-091 (corrupt patch in
worktree import — worker imports used checkout fallback) and ISS-019
(plan.toml-not-found on provisioned worktrees). Neither is a regression — the
mechanical extraction of `coordinate()` faithfully preserves existing behavior.
Both have backlog items tracking the root causes.

### Tradeoffs consciously accepted

- **Skill verbosity.** The dispatch router at 45 body lines is terse but
self-contained. The arm skills at 29 and 24 lines are thin spawn templates. A
future agent will need to consult `dispatch --help` for flag details — this is
by design (the CLI is the source of truth).
- **No file-disjointness in v1.** The plan.toml schema has no `files` field per
phase. The orchestrator still runs `/phase-plan` for each candidate phase — the
omission is scoped out (design D5).
- **Checkout fallback for worker import.** Functionally equivalent for code
correctness, but worker forks lose the import→land→gc pipeline path. The
tradeoff was "import bug blocks dispatch entirely" vs "checkout works but loses
GC" — the latter is clearly correct.

## Reconciliation Brief

### Per-slice (direct edit)

No design or governance changes are needed for SL-085. All 4 findings are
tolerated or aligned — no spec, design, or governance artifact requires
audit-driven correction.

- **F-1 (dispatch/085 absent):** Process recordkeeping gap — tolerated. The
dispatch ref lifecycle is a standing risk, not a design error. The code is
correct.
- **F-2 (line count ceilings):** Aligned — within design targets.
- **F-3 (ISS-019):** Tolerated — pre-existing bug, faithfully preserved by
extraction.
- **F-4 (IMP-091):** Tolerated — pre-existing bug, checkout fallback is a
conscious tradeoff.

### Governance/spec (REV)

None. No ADR, spec, or governance document requires amendment as a result of
this audit.
