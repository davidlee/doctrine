---
name: dispatch-spawn
description: The one spawn path for `/dispatch` — `scripts/spawn-confined.sh <harness>` forks the worker's worktree, wraps it in a kernel-level jail, and runs the harness headless inside it. Every harness takes this path; there is no in-session arm and no unconfined fallback. The worker cannot commit — it hands back an uncommitted working tree and the orchestrator imports it. Reached from the `/dispatch` router; do not invoke directly.
---

# Dispatch — spawn a confined worker

One spawn shape, for every harness. The drive cadence lives in the
[`/dispatch` router](../dispatch/SKILL.md); this skill is the spawn call plus the
landing mechanics.

**The worker is confined by the OS, not by cooperation.** Inside the namespace
only its own worktree and the harness's config dir are writable; everything else
— the coordination tree, sibling forks, `.doctrine/`, and the real git dir — is
read-only. So the worker **cannot commit at all**, and its hand-back is an
*uncommitted working tree* that the orchestrator imports. Never instruct a worker
to commit.

## Spawn

```sh
scripts/spawn-confined.sh <harness> <B> <BRANCH> <DIR> <PROMPT_FILE> [BACKSTOP_SECS]
```

`<harness>` is `pi` or `claude`. The script does the whole pre-spawn sequence
itself — you do not fork separately:

1. **Capability probe.** Linux asserts `bwrap` is present and refuses
   `bwrap-unavailable` *before* minting a fork; macOS gets its named refusal from
   `doctrine worktree jail-prefix`, which fails closed and writes no prefix.
   **There is no unconfined fallback** — an unresolvable jail aborts the spawn
   rather than degrading (an empty prefix is caught by its own guard).
2. **Fork.** `doctrine worktree fork --base <B> --branch <BRANCH> --dir <DIR>
   --worker`, orchestrator-classed, run from the project root before any
   confinement. The fork is deliberately **unbound** — no `--slice`/`--phase`
   (see *The funnel*, below).
3. **Confine + exec.** `timeout <BACKSTOP> <jail prefix> <harness exec>`,
   delivering `<PROMPT_FILE>` to the worker (stdin for `claude`; one compact RPC
   line for `pi`) and capturing the transcript. The jail rw-binds the fork dir
   and the harness config dir, ro-binds the rest, and sets `DOCTRINE_WORKER=1`
   inside the namespace.
4. **Hand back.** `claude -p` exits when the turn ends — process exit *is* the
   completion signal. `pi --mode rpc` never self-exits, so that profile alone
   holds stdin open with a fifo and polls for its typed settle event. That
   asymmetry is the only harness difference beyond the config dir and the exec
   line.

**The prompt is pre-distilled and self-contained** — the worker inherits no
conversation. It must state that the fork is a **clean** checkout with no WIP,
forbid every work-discarding git verb (`reset`, `checkout --`, `stash`, `clean`
— a worker that hallucinates "pre-existing WIP" can silently destroy its own
delta), and instruct the worker to leave its work **uncommitted** in the tree.
For a TDD red proof, tell it to *edit* the scratch out, never to git-discard it.

Never trust the worker's self-reported success. The tree is the ground truth.

## Landing

The delta is an uncommitted working tree that **persists** after the process
exits — nothing auto-reaps it.

1. **Import.** `doctrine worktree import --base <B> --from-worktree <DIR>
   --slice <N>` — gathers the tracked + untracked delta, runs the
   `classify_import` scope belt as a HARD pre-apply gate (`.doctrine/` /
   `.claude/` reject, undeclared-scope reject), applies onto `B`
   **NON-committing**, then runs the reject-and-halt prove gate in-process. An
   unformatted or lint-red delta HALTS staged and is **never** auto-fixed: that
   is a worker-delta defect, distinct from a dirty-base defect. Any refusal is
   report-and-halt — never import around it.
2. **Commit ONE** on the coordination branch — the orchestrator's own act, and
   the reason import is non-committing.
3. **Record the boundary.** `doctrine slice record-delta <N> PHASE-NN --commit
   <S>`, with `S` **pinned at the code commit**, never read from `HEAD` at record
   time. Knowledge commits trail the code commit, so `HEAD` may have moved;
   `--commit` records exactly `S`'s own patch, which excludes trailing knowledge
   and any refresh-base merge by construction. This write is **not skippable** —
   `dispatch sync --prepare-review`'s completeness gate halts if a completed
   phase has no registry row.
4. **Verify and conclude.** Run the phase's verification on the coordination
   tree (the worker's own report is advisory), then flip the sheet:
   `doctrine slice phase --status completed <N> PHASE-NN`.
5. **Reap** the spent fork: `doctrine worktree gc --fork <BRANCH>` — the
   unfunnelled-fork case its patch-id oracle is narrowed to, and idempotent.
   Never `git worktree remove --force` an unimported tree: that deletes the sole
   copy of the delta. `--dry-run` prints the verdict and destroys nothing.

## The funnel: a bound fork's machinery, not this path's

`doctrine dispatch next` prescribes from a **committed funnel row**, and a row
only exists for a fork bound at creation (`fork --worker --slice N --phase
PHASE-NN` under `<coord>/.worktrees/<name>`). This path forks **unbound**, so no
`Spawn` row ever lands and `next` sits at `spawn` indefinitely.

That is not a defect and not something to heal: this is the **main-thread
orchestrator** — it applies the delta, commits, records the boundary and flips
the phase as separate acts, and never consults the funnel record. The funnel
machinery is retained and unchanged; it simply does not describe this drive.
Read `next` as advisory here, not as the driver.

## Red Flags

**Never:** instruct or expect a worker to commit (its `.git` is read-only —
there is no self-commit path); spawn outside the script (a hand-rolled exec is
an unconfined worker); omit the backstop timeout; `eval` the prompt; use a
heredoc for the pi profile's RPC stdin (EOF kills it); import around a scope or
prove-gate refusal (a refusal is a defect or a halt, never a detour); auto-fix a
red delta into the import; force-remove a fork before its delta has landed;
trust the worker's own green/failure labels; treat a `spawn` prescription from
`dispatch next` as a stuck funnel.

**Always:** halt on a non-zero spawn (fail-closed is the whole posture — a
confinement that cannot be established means no worker); pass a pre-distilled,
self-contained prompt; import with `--slice` so the scope belt has selectors to
check; pin `S` at the code commit before recording the boundary; return to the
router for the drive cadence.
