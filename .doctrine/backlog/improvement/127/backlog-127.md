# IMP-127: dispatch candidate: ingest a hand-resolved 3-way merge conflict

## Problem

`doctrine dispatch candidate create` is all-or-nothing: it runs its own internal
3-way merge and either records a clean candidate, or — on *any* conflict — parks
the worktree at base with `status=conflicted, merge_oid=""` and stops. There is
**no verb to feed a manual resolution back in.** Once conflicted:

- `candidate admit` refuses ("no Doctrine merge to validate").
- re-running `candidate create` recomputes the same merge and re-conflicts.
- resolving + committing in the parked worktree and `git checkout -B`-ing the
  branch does *not* help — admit validates the recorded `merge_oid`, which is
  still empty.

So the agent dead-ends, even though the underlying git conflict is often trivial
to resolve by hand (observed SL-104 close: an add/add on a test file both main
and the bundle created independently — resolve = take the matured version, 30
seconds in plain git).

The deliberate decision was **no `--force`** (don't bypass the merge) — correct.
The gap that leaves is the *opposite* of force: an **"it's complicated" path**
that lets the agent do the real 3-way merge by hand and have the tool **adopt the
resolution** as the candidate's Doctrine merge.

## Why it matters

Trigger is *base drift*: whenever trunk moves between bundle creation and close so
the close_target auto-merge conflicts. Not exotic — split lineage (a phase landing
on main while a bundle is in flight), a sibling slice closing first, a dirty-tree
rescue commit all produce it. Expect it reasonably often. Today the only escape is
to abandon the admitted-OID CAS provenance and direct-land on main (SL-104,
user-approved) — forfeiting exactly the integrity the candidate seam exists to give.

## Sketch (not a design)

A verb that adopts the parked worktree's resolved commit as the candidate merge:
validate it is a genuine merge of (base, source) — both its parents, base/source
OIDs match the recorded row — then record `merge_oid` from the committed tip and
flip `status` so `admit` accepts it. The provenance check `admit` already runs
(recorded merge is a Doctrine candidate merge + ancestor of the admitted tip) is
the contract to satisfy; this just lets a *hand-made* merge satisfy it instead of
only an auto-made one. No `--force`, no bypass — the merge still happens and is
still validated; the operator just performs it.

Open questions for design:
- New verb (`candidate resolve`?) vs. a flag on `create` that adopts an existing
  resolved parked branch.
- Validation strictness (parents + base/source OID match is the floor; content is
  the operator's call by definition).
- Interaction with the zero-OID CAS guard on the candidate ref.

## Provenance

Surfaced at SL-104 close. See memory
`mem.pattern.dispatch.split-lineage-close-conflict-direct-land` for the full
incident and the direct-land workaround.
