Four rules for what an orchestrator puts in a sub-agent's brief. All four were
learned by a brief being wrong and the worker catching it — or not.

## Label the diagnosis a hypothesis, and give constraints instead of a route

A `SL-248` brief carried an orchestrator diagnosis that was wrong in its
specifics and prescribed a repair route that measurement then rejected. Both
were caught only because the brief said *verify it, don't take it on trust* —
and the worker did.

Keep that phrasing on every diagnosis you hand down. Never prescribe a route as
settled: give the constraints the fix must satisfy — here, *don't weaken an
assertion, don't reopen a settled decision, don't launder a flake into a smaller
flake* — and let the worker choose against them. **A worker that can overturn
the brief on evidence is the point of the split**; a brief that forecloses that
has thrown away the second opinion it paid for.

## The orchestrator's own verification run is what convicts

Not the worker's self-tally. Both times a worker's honest tally was
contradicted, it was contradicted by the first run taken outside the worker's
own process and sitting. Never skip that run because the hand-back looks strong
— a strong tally is exactly when it is worth taking. What makes a tally
untrustworthy even when honestly gathered:
[[mem.pattern.testing.convict-a-race-by-causality-not-repetition]].

## Name any task that structurally cannot stage a red

...and the compensating positive control that must fail in its place. A test
written after the code that makes it compile passes on its first run and proves
nothing. The planner owes this per task, not per phase.

## Require inventories to be verified against the tree

Never against a prior sheet, a brief, or a recollection. "The five touch sites
for a new `Finding` category" were six, and the sixth silently dropped data.

## And read a diagnostic as liveness before you read it as a defect

The diagnostic feed shows a live worker's **uncommitted intermediate states**. A
worker mid-refactor has helpers landed and call sites not yet moved, so
`dead_code` and unresolved-name diagnostics are the expected shape of work in
progress. Before advising on one, check the condition survives into a commit —
`git show <tip>:<path>` — because only a committed state is a claim about
anything. An advisory that turns out to be noise costs the worker a cycle and
costs you the credibility of the next one, which may be real.
