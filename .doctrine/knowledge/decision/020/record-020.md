# DEC-020: Non-contribution classification leaves SL-230

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The decision

`memory verify` **attests over every non-contributing scope entry**, reports each
on stderr, and `validate` raises them as corpus-health findings. It does **not**
classify them, and it refuses on none of them.

The question *should non-contribution refuse, and on which entries* leaves SL-230
and goes to its own slice.

Taken by the user at RV-307 round 7, on the responder's recommendation.

## Why — three instruments, three refutations

D10's stale-versus-unobservable boundary was drawn three times, and each
instrument was refuted because it reads **local repository state**:

| Round | Instrument | Refuted by | Because |
|---|---|---|---|
| 4 | blanket refusal (no boundary) | F-21 | 36 active items stop verifying |
| 6 | filesystem existence | F-25 | checkout-dependent — `.claude/skills/**` is absent from a source checkout, present in an installed one |
| 7 | `git rev-list --all` history | F-31 | ref-set-dependent — deleting a branch flips a once-tracked path from refuse to attest while its commit object survives |

Verified (git 2.54.0): with the branch present `rev-list --all --max-count=1 --
<path>` returns 1; after `git branch -D`, 0, with `cat-file -e` still succeeding.

The pattern is not three unlucky choices. A shallow clone, a pruned repo, a fresh
clone and a dispatch worktree legitimately hold different answers, because **git's
view is inherently local**. So the distinction is not reliably decidable from
repository state, and a fourth derived instrument fails for the same reason as the
first three.

## What this does and does not concede

It does **not** concede F-25's substance. F-25's force was that the design
*condemned* the stderr advisory ("a warning does not make an unobservable claim
committed") and then quietly adopted it for two of four classes. Adopting it
explicitly, with the weak reading stated and the residual routed, is a stated
position rather than an inconsistency.

It does concede that `verified_sha` takes the **weak reading**: it asserts that
everything git could observe about the claim was committed and unchanged at that
commit, not that every declared entry was observed. That reading must be the
*only* one in the design — RV-307 F-33 found the strong reading still asserted at
`design.md:606` and in `slice-230.md`.

## The stable answer, deferred

A boundary that survives cloning must be **declared**, not derived — a signal on
the record marking evidence git is not expected to observe. That is a schema
change, which is the same shape and cost as IMP-318 (persist attested coverage)
and QUE-173/OQ-3 (body digest). All three should be scoped together: each exists
to make an attestation say more than a sha.

## Consequences for SL-230

- Zero memories lose a stamp (the round-6 cut would have cost 4).
- D10 shrinks to: malformed/probe-aborting entries refuse (E11, E13 — they abort
  the probe, which is a mechanical necessity, not a classification); everything
  else that does not contribute is reported and attested over.
- RV-307 **F-31** and **F-32**'s history-probe limb dissolve — there is no history
  probe. F-32's prefix-splitting defect survives and still needs fixing.
- **F-33** is fixed by deleting the strong reading, not by weakening the quote.

## Relations

- SL-230 (design D10, R8), RV-307 (F-6, F-21, F-25, F-31, F-33).
- IMP-318 — persist attested coverage; the residual this decision leaves open.
- QUE-173 — digest-based invalidation; same schema-change class.
