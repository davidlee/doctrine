# Dispatch phase-status is per-tree runtime; prepare-review reads the primary tree

Phase-completion status lives in **per-worktree gitignored runtime state**
(`.doctrine/state/slice/NNN/phases/phase-NN.toml`), written by `doctrine slice
phase … --status completed`. It is NOT a committed/shared artifact — each worktree
has its own copy.

## The split-brain
A phase executed via `/dispatch` from the **coordination** worktree flips
`completed` only in the coord tree's runtime. Phases run earlier from the
**primary/root** tree flipped it there. So the two trees hold *inverse* views of
which phases are done.

`dispatch sync --prepare-review` and `slice conformance`'s completeness gate read
the **primary tree's** runtime. A phase you completed in the coord tree therefore
shows as NOT completed to prepare-review, which halts:

```
prepare-review: conformance registry incomplete:
  recorded row for PHASE-NN, which is not a completed phase
```

(A registry row exists — `record-delta` wrote it — but the phase-status the gate
reads says `planned`.)

## Fix
Flip the phase in the tree prepare-review reads (the **primary/root** tree, run
from the session root, not the coord worktree):

```bash
doctrine slice phase SL-NNN PHASE-NN --status completed --note "flip missed in coord-tree dispatch run"
```

Then re-run `dispatch sync --prepare-review`. Flipping it in the coord tree alone
is wasted — the coord runtime is disposable and removed at conclude.

## Symptom to recognise
`slice status SL-NNN` in the coord tree disagrees with prepare-review on the
completed-phase set. Cross-check both trees' `phases/phase-NN.toml` before assuming
a real gap.

Surfaced during SL-186 close (RFC-011 case-notes `[conclude; SL-186-P04-conclude]`).
