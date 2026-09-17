An oubliette capsule (ADR-020) runs on a linked worktree and hands back an
**uncommitted working tree**; the orchestrator imports the *diff*. Phase sheets
live in `.doctrine/state/slice/<N>/phases/`, which is **runtime tier and
gitignored** (ADR-003 storage rule), so nothing under it is in that diff.

Consequence: every `status = "completed"` flip, and every worker note written
into `phase-NN.md`, stays in the fork. The coord tree keeps the `planned`
scaffolds `phase-plan` materialised. `doctrine slice status <N>` then reports
`phases: 0/N` on a slice that is implemented, audited and reconciled — a reading
indistinguishable from an unstarted slice. Nothing in the hand-back flags it.

Observed on SL-245: coord held 1.2–1.4 KB `planned` scaffolds while
`.worktrees/SL-245-c11` held 26–51 KB `completed` sheets. `/close`'s rollup
pre-check is where it bites, because that gate reads the rollup.

**Fix.** Before `/close`, copy the fork's sheets over the coord tree's:

```bash
cp .worktrees/<fork>/.doctrine/state/slice/<N>/phases/phase-*.{toml,md} \
   .doctrine/state/slice/<N>/phases/
```

This restores a fact rather than authoring one — the fork's sheets are the
record of what the workers actually did, and runtime tier is disposable by
design. Verify with `doctrine slice status <N>`: it should read `<N>/<N>` with
the expected `⚠ divergent: phases complete but lifecycle not terminal`, which is
exactly what `/close` then resolves.

Do **not** hand-edit the coord scaffolds to `completed` — that fabricates the
status and discards the workers' notes. Copy the real sheets.

Distinct from `/dispatch`'s trunk-row requirements: a capsule slice has no
dispatch journal at all, so `/close` steps 3a/3b do not apply to it.


## Related

The same split-brain in the dispatch path, where it bites at prepare-review
rather than at close:
[[mem.pattern.doctrine.dispatch-phase-status-per-tree-split-brain]],
[[mem.fact.dispatch.prepare-review-reads-primary-phase-status]].
