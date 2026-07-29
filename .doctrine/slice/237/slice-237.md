# Primary-resolve runtime phase state

## Context

Runtime phase sheets (`.doctrine/state/slice/NNN/phases/phase-NN.toml`) resolve
against whatever root the caller hands them; the source-delta registry
(`boundaries.toml`), one directory level away, resolves against the **primary
worktree**. Two files describing the same slice, in the same tier, with different
homes:

- `boundaries_path` (`src/state.rs:716`) → `crate::git::primary_worktree(cwd)`,
  documented *"so every worktree shares one file"*.
- `phases_dir` (`src/state.rs:135`) → a bare path join.

Nothing in the naming or types signals which you get. The divergence was never
decided; it is two call sites that chose independently. It is the single root
cause behind a five-patch history:

| | |
|---|---|
| **ISS-212** (closed) | orchestrator-author flips completion from a coord cwd; `prepare-review` reads primary sheets. Fixed by a completion mirror. |
| **IMP-272** (resolved) | the claude-arm instance; its inline mirror was later folded into ISS-212's. |
| **IDE-028** (open) | the mirror is `completed`-only — `in_progress` / `blocked` / reopen stay primary-invisible. |
| **ISS-269** (open) | `slice conformance` reads boundaries from primary and phase status from cwd, so a linked worktree reports the exact inverse of the primary. |
| **SL-228 / RV-312 F-6** | a phase appended mid-drive has *no* primary sheet; the mirror cannot create one and `dispatch sync --prepare-review` hard-refused two fully-landed phases. |

Each patch treated an instance. This slice removes the generator.

**The unrecognised axis.** "Authored / runtime / derived" says how *durable* a
file is. It says nothing about **whose fact it is**. Within the runtime tier:

| Scope | Meaning | Examples |
|---|---|---|
| **repo** | true of the *slice*, whichever tree you stand in | `boundaries.toml`, `phases/` |
| **tree** | true of *this checkout* | `boot.md` |
| **session** | true of *this agent run* | `handover.md`, `mem-surface-seen-<uuid>.txt` |

"PHASE-03 is completed" and "PHASE-03 spans oids X..Y" are the same kind of fact
— repo-scoped, one home. Phase sheets are simply mis-scoped.

**Governance.** ADR-006 Context force 3 names *"gitignored runtime state is
invisible across worktrees"* as a **hazard**, so this slice serves the ADR's
problem statement rather than contesting a decision. ADR-006 D9 (workers write no
doctrine-mediated state — all writes funnel through the orchestrator) means no
worker-side phase-status writer exists, so primary-resolving raises no worker
confinement problem.

*Amended in `/design` (2026-07-29).* The exposure is **decision-level, not merely
context drift**. Three statements are falsified — ADR-006 **D2**'s parenthetical
that the coordination/runtime tier is *withheld* from workers, ADR-006 **D4**'s
clause that *runtime state stays gitignored/per-worktree*, and **SPEC-012** §
"Tier merge-safety by construction", which defends D4 *"not by trust but by
absence"*. D2 and D4 are Decisions. One REV covers all three (ADR-013); see
`design.md` § 7. It is a **restatement, not a relaxation**: merge safety is not
weakened (no copy ⇒ nothing to diverge), REQ-297's write prohibition stays
enforced at `src/commands/guard.rs:78,80`, and on the subprocess arm the
prohibition additionally gains an OS floor.

## Scope & Objectives

1. **Make runtime-state scope explicit at the path constructor**, not ad hoc per
   call site. Repo-scoped state resolves through `primary_worktree`; the choice is
   visible in the API rather than inferable only by reading the body.
2. **Primary-resolve phase sheets** under that rule, with a non-repo fallback
   (bare/not-a-repo cwd degrades to the given root — `phases_dir` is total today
   and has far more read callers than write callers, including tests on plain
   tempdirs).
3. **Retire the completion mirror.** `mirror_completion_into_primary`
   (`src/state.rs:518`) and its live-coord/solo-fork guards become dead once both
   trees name one file. Closes IDE-028's `completed`-only residue by dissolution,
   not by widening it.
4. **Dissolve the appended-phase refusal.** `slice phases` run from a coord tree
   reads that tree's `plan.toml` and writes the primary state dir, so a
   mid-drive-appended phase materialises where the completeness gate reads it.
5. **Narrow SL-190's composite** *(corrected in `/design`; this objective
   previously read "retire the composite", which was wrong)*.
   `resolve_phase_truth` **narrows, it does not die**: it loses the
   coord-vs-local axis and **keeps landed-vs-sheet**, retaining the `Some`-branch
   matrix so conflict detection survives. `slice reconcile-phases` keeps working —
   its real job is rewriting sheets from the landed oracle, which this slice does
   not touch. `slice status --across-trees` is renamed **`--truth`**. See DEC-096.
6. **Migrate `boundaries_path` onto the same mechanism** (`src/state.rs:716`), so
   the slice does not ship two primary-resolution mechanisms in one module. It
   becomes infallible. See DEC-098.
7. **Mint the `phases` convenience symlink in the primary only**
   (`src/state.rs:345`), so a linked worktree has no link rather than a dangling
   one. See DEC-097.

Closure intent: ISS-269's reproduction inverts — `slice conformance <ID>` agrees
from a linked worktree and the primary. IDE-028 and the RV-312 appended-phase case
close as absorbed. The behaviour-preservation gate holds: existing suites stay
green unchanged except where they encode the two-copy model.

## Non-Goals

- **Other runtime subtrees.** `.doctrine/state/dispatch/` and `.doctrine/state/review/`
  are suspected of the same defect; their resolution is unverified and each has its
  own concurrency story. Audited separately under **CHR-050**.
- **The session tier.** No defined home, no reaper (IMP-338); the flat-drawer
  accumulation is real but out of scope.
- **The completeness gate's diagnostics.** ISS-254 (no evidence-only-phase
  exemption; refusal cannot name which input disagreed) survives this change
  untouched.
- ~~**Amending ADR-006.**~~ *Retired in `/design`.* The condition this non-goal
  set — *"unless `/design` finds a genuine decision-level conflict"* — was met:
  ADR-006 D2 and D4 are Decisions, and both are falsified. A REV covering ADR-006
  and SPEC-012 is opened **in this slice**, not deferred to reconcile; landing
  code that knowingly falsifies an accepted Decision while the ADR still asserts
  it would reproduce this slice's own defect in governance.
- **Reworking the storage-tier vocabulary.** The repo/tree/session axis is design
  rationale here. Promoting it to governance is a later, separate call.
- **Fencing worker *read* access to primary phase sheets.** Single-homing makes
  the runtime tier readable from a fork. Accepted explicitly rather than fenced
  (`design.md` § 7 D5): ADR-006 D2 already grants workers free read of the
  authored tier, which includes the slice design — strictly more sensitive than a
  phase sheet's status string, so this adds no new *class* of information. The
  residual is prompt discipline, not an invariant breach, and it is named in the
  REV.

## Summary

## Follow-Ups
