---
seq: 0017
scope: codebase
target: `doctrine worktree gc` (SL-056 PHASE-09, design §8)
confidence: med
reversible: yes (proposal only; read-only analysis — nothing built or reaped)
---
## What
`doctrine worktree gc` reaps spent forks safely but **only one at a time** — it
requires `--fork <FORK>` and reaps that single fork iff it provably landed (the
two-leg ancestry ∪ patch-id oracle, §8.1). There is **no bulk/sweep mode**
(`gc --help` flags: `--fork` (required), `--superseded-head`, `--force`,
`--dry-run`, `--path` — no `--all`).

At parallel / team scale this is the wrong granularity. This very repo right now
holds **77 worktrees** (`git worktree list`): 35 under `.worktrees/`, 19
`dispatch/candidate`, 16 `worktree-agent-*`, plus one already flagged `prunable`.
Reaping accumulated spent forks today means an operator enumerates forks by hand
and runs `gc --fork …` N times — 77 invocations to clear the backlog of landed
ones. No backlog item tracks this (a `backlog list` scan for worktree-gc/sweep/
reap/accumulation finds only unrelated dispatch issues).

The safety machinery to do this well **already exists**: the landing oracle is the
hard part and it's shipped, idempotent, and fail-closed. A sweep is pure
composition over it — `for each fork: run the existing oracle; reap iff landed;
report the rest with their refusal token` — adding no new destructive logic, only
iteration. This is the same reuse posture as the other proposals (0003 cordage
`reachable`, 0011 the four checks): the primitive is done; the missing piece is the
batch surface over it. A `gc --all --dry-run` would also be a high-value *operator
view*: "which of my 77 worktrees are provably reapable right now?" — answerable
today only by 77 manual dry-runs.

Caveat (load-bearing): the per-fork design is plausibly **deliberate** — each reap
is destructive and gated by a proof, and a bulk sweep destroys many at once. So a
sweep must inherit, not weaken, the oracle: default to a dry-run-style report,
reap only provably-landed forks, and refuse to combine with `--force` in bulk.

## Options
1. **`gc --all` (oracle-gated sweep).** Iterate all forks (or a filtered set),
   reap each that the existing oracle certifies landed, print a per-fork verdict
   table for the rest. Default `--dry-run`; `--all --force` explicitly disallowed
   (or loudly confirmed). Tradeoff: closes the 77-worktree operability gap with
   zero new safety logic; the only design work is iteration + the verdict report
   and a scope filter.
2. **`gc --all --dry-run` only (report, never bulk-reap).** Ship just the operator
   *view* — "here's what's reapable" — and keep actual reaping per-fork. Tradeoff:
   captures most of the value (visibility into the pile) at near-zero risk; still
   N invocations to actually clear, but now an informed N.
3. **Leave per-fork.** Tradeoff: zero effort; but parallel/team usage (the whole
   point of the dispatch/worktree machinery — PRD-015, ADR-006/008/012) accumulates
   worktrees with only a one-at-a-time broom. 77-and-counting is the evidence.

## Recommendation
Option 1 with Option 2 as its default mode: build `gc --all` that defaults to the
dry-run verdict report and only reaps with an explicit confirm, always through the
existing oracle, never bulk-`--force`. Rationale: the dangerous part (deciding a
fork is safe to delete) is already solved and reused unchanged; what's missing is
the batch ergonomics that parallel work demands. Shipping the report-first default
honours the deliberate per-fork caution while removing the 77×-toil.

Decisions deferred to YOU:
- (a) **is per-fork-only deliberate** (safety: never mass-delete), or just unbuilt
  batch ergonomics? If the former, ship Option 2 (report) only.
- (b) **scope filter** — sweep all forks, or class-scoped (dispatch-candidate vs
  worker vs solo `.worktrees`)? The 19 dispatch-candidates may have a different
  lifecycle than the 16 agent worktrees.
- (c) **who runs it** — orchestrator-classed like single `gc` (refused under
  worker-mode), presumably yes.

## Next doctrine move
```
# confirm the gap + the pile (read-only):
doctrine worktree gc --help          # --fork required; no --all
git worktree list | wc -l            # the accumulation (77 here)
# (operator could already, tediously, dry-run each fork to see what's reapable)

# capture the batch surface (NOT executed — fence forbids backlog transition):
doctrine backlog new improvement "worktree gc --all: oracle-gated bulk sweep over \
  all/filtered forks, dry-run-default verdict report, reap only provably-landed — \
  composition over the existing SL-056 §8.1 landing oracle; no new destructive \
  logic. Parallel-scale operability (77 worktrees accreting today)" \
  --tag area:worktree --tag area:dispatch --tag cli
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — the value is "iterate the existing oracle + report," not a diff; the per-fork
reap code (SL-056 §8) is the reused body, and the safety question (a) is the real
gate, which a speculative diff would prejudge.
