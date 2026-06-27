# ISS-056: Stale-base dispatch integrate silently deletes authored corpus

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Summary

A `/dispatch` drive whose coordination branch is forked from a base that
**predates the authored `.doctrine` corpus** produces phase commits carrying a
tree with **no corpus**. Integrating/merging that bundle onto the trunk
**silently deletes the entire authored corpus** (all ADRs, slices, specs,
standards, RFCs, backlog) with no conflict, no abort, no diagnostic. Witnessed
live on the SL-164 drive (2026-06-27): 4816 authored files vanished from `edge`
and `main`; only the SL-164 source delta survived.

This is the **silent, catastrophic** member of the stale-base family. The loud
variants are already handled — RSK-010/SL-127 (candidate-time conflict,
mid-drive refresh) and ISS-036 (setup hard-abort when the base predates the
slice's *own* plan). Those fail *closed*. This one fails *open and destructive*.

## The failure chain (SL-164)

1. The edge/main split: `main` is the dispatch landing zone; `edge` carries
   authored content. Per the AGENTS.md pre-dispatch ritual, `git fetch . edge:main`
   promotes edge→main. Here the sequence left `main` fast-forwarded to **`caf8dfc6`**
   — a fork base that predated the corpus on that lineage.
2. `dispatch setup` forked the coordination worktree at that stale base. The base
   contained the SL-164 *plan* (so SL-127/ISS-036's hard-abort did **not** fire) but
   **not** the broader authored corpus.
3. The phase commits (`c9a9da0c`, `da807bb6`, `08bbbfa5`) were authored on that
   tree. Each carries the SL-164 source delta on top of a corpus-less tree.
4. `dispatch sync --integrate` / the merge onto edge (`db7fca00`) replayed that
   tree onto trunk. Because the bundle's tree simply *lacks* the corpus, the merge
   recorded it as a clean **deletion of 4816 files** — no conflict to stop on.
5. Subsequent hand-merges propagated the gutted tree across `edge`↔`main`. Partial
   manual restores (`1baa3b71`, `2a247af1`) clawed back only the slice's own files,
   masking the scale of the loss.

Recovery: restore the corpus from the last good tip (`81b6b4f4`), re-layer the
genuine post-tip work, rebuild a clean linear history. Nothing was permanently
lost (intact in the reflog and on `origin/edge`), but the corpus was absent from
the working trees and both branch tips until repaired.

## Why it warrants a fix (distinct from ISS-038 and SL-127)

- **Mirror of ISS-038, not a duplicate.** ISS-038 = a *dirty shared checkout* →
  phantom index → silent revert of **code** (docs survived). This = a *stale fork
  base* → bundle tree missing corpus → silent deletion of the **corpus** (code
  survived). Same outcome shape (integrate silently deletes committed content),
  opposite halves, different mechanism and different fix.
- **SL-127 closed the loud variants only.** Ancestor-dominant ladder + mid-drive
  refresh stop the *conflict* and *hard-abort* paths. They do not detect a base
  that is fresh enough to hold the slice's plan yet stale enough to be missing the
  rest of the corpus — the exact gap that bit here.
- **Integrate/merge must fail-closed on corpus-shrinking projection.** A pre-gate
  before advancing trunk: if the projected tree *deletes* authored `.doctrine`
  paths that the integration did not explicitly author, **refuse**. A bundle should
  never be able to remove authored entities as a side effect of a stale base.
- **The edge/main promotion ritual is a footgun.** `git fetch . edge:main` can
  fast-forward `main` to a point *behind* the authored corpus when the split is
  mismanaged. The ritual needs a guard (or to be checkout-independent) so promotion
  cannot land main on a corpus-predating base.

## Candidate fixes

- (a) **Deletion guard at integrate**: refuse to advance trunk if the projection
  removes authored `.doctrine/**` paths not in the slice's own authored set.
- (b) **Base-corpus freshness check at setup**: assert the fork base contains the
  *current* authored corpus head, not merely the slice's plan.
- (c) **Promotion guard**: `edge:main` fast-forward must be ancestor-checked
  against the authored-corpus tip, not just ref reachability.

Related: ISS-038 (mirror — silent code revert via phantom index),
RSK-010 + SL-127 (stale-base loud variants, fixed), ISS-036 (setup stale-base
hard-abort), RFC-005 (dispatch funnel integrity hazard survey), ISS-030
(phantom reverse-diff detector), IMP-174 (split-brain authored state).
