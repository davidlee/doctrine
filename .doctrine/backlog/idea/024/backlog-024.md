# IDE-024: Confined claude worker via standalone clone: worker self-commits, orchestrator cherry-picks

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Alternative confinement topology for the claude dispatch arm, deferred from
**SL-182** (which lands the proven **linked-worktree + ro-shared-`.git`** path,
"Path L"). This is "Path C". Prioritise on **observed orchestrator cost**, not
speculatively.

## The mechanism

A confined worker on a **linked** worktree cannot self-commit: its object store is
`<main>/.git/objects`, outside the rw worktree bind, ro under `--ro-bind / /`
(RSK-014, CLAUDE.md "worker CANNOT self-commit"). Give the worker a **standalone
clone** instead — `.git` lives *inside* the worktree dir `$D` → rw under the
unchanged bwrap profile (`--ro-bind / /` + `--bind $D $D`), while the main repo's
`.git` stays ro. Confinement is **fully intact** (main store uncorruptable); the
worker commits into its own isolated store. `git clone --local` hardlinks existing
objects (cheap disk); new commits land in the clone. Orchestrator imports via
`git fetch <clone> <branch>` + cherry-pick.

## Why (the real value axis — efficiency, NOT commit aesthetics)

The point is **orchestrator time & token efficiency**, not "real commits" for
their own sake. On the ro-`.git` arm (Path L, and today's pi arm) the worker
cannot run a commit-gated self-verify, so the **entire verification burden falls
on the orchestrator** — it must re-run the full suite on the imported tree because
it cannot trust the worker's self-reported green (empirically hollow: pi
PHASE-01/02 both reported green falsely, RSK-014 / case-notes SL-171). A
self-committing worker can `just check`-then-commit autonomously and hand back a
**worker-verified** delta, letting the orchestrator do lighter confirmation
instead of a full from-scratch re-run. The saving compounds across phases and
parallel workers.

## Costs / unknowns (why it's deferred, not done)

- **Topology change** — `doctrine worktree create-fork` must produce a clone, not
  a linked worktree, for the confined claude arm. Likely **ADR-altitude** (worktree
  topology for confined workers; touches ADR-006 / ADR-008 / ADR-012).
- **Hook binding-validation** — a clone is not `is_linked_worktree`; the SL-182
  PreToolUse handler must recognise the clone as a valid jail target.
- **Funnel import** — `fetch + cherry-pick` instead of working-tree-diff apply.
- **Harness tolerance (spike first)** — does the claude harness tolerate a clone
  where its `WorktreeCreate` hook created a linked worktree? Unproven; verify
  empirically before committing (cf. verify-harness-behavior-empirically).
- **Unproven end-to-end** — the SL-182 probe validated the bwrap *write wall*, not
  the dispatch funnel; Path C is a new topology on top of that.

## Decision basis

Land SL-182 (Path L) first; it converges the claude arm onto the proven pi funnel
(working-tree-diff import). Pull this idea forward only if observed orchestrator
verify-cost (full-suite re-runs, hollow-green re-dispatches) justifies the topology
change. Recommend a `/consult` to scope ADR-vs-slice + a harness-tolerance spike at
that point.

Refs: SL-182 (Path L, the chosen close), RSK-014 (the risk), ADR-006/008/012
(worktree topology), `case-notes.md` SL-171 (pi self-commit / hollow-green
evidence).
