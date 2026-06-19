---
seq: 0013
scope: spec
target: SPEC-005 (ADR entity surface)
confidence: high
reversible: yes (proposal only; no authored-tier edit — fence holds)
---
## What
SPEC-005's description of the ADR supersession storage is **stale on three counts** —
it describes the pre-migration world and was never updated after SL-095 / SL-062.

What SPEC-005 says (`.doctrine/spec/tech/005/spec-005.md`):
- `:46-47` — "a typed `[relationships]` table holding the supersession **pair**
  `supersedes` / `superseded_by`".
- `:49-50` — "The supersession **pair stays typed** pending the transactional
  supersede verb (`IMP-006`)."
- `:81-84` (D2) — "the supersession pair + `tags` remain a typed `[relationships]`
  table, **inert until** the transactional supersede verb (`IMP-006`) wires them."

What the authored ADR TOML actually says (`.doctrine/adr/004/adr-004.toml:12-15`,
identical in `adr-012.toml`):
- "The `supersedes` and `related` axes are **uniform `[[relation]]` rows now
  (SL-095, SL-048)**." → `supersedes` is **no longer typed**; only `superseded_by`
  (the ADR-004 §5 derived-reverse carve-out) and `tags` stay typed.

And the verb state: `IMP-006` is **resolved/fixed** (SL-062), and `doctrine
supersede` is a live command (it writes `supersedes`/`superseded_by`/flips status).

So three drifts:
1. **`supersedes` typed → actually `[[relation]]`** (migrated SL-095). SPEC-005
   still groups it with `superseded_by` as "the typed pair."
2. **"inert until the verb wires them" → the verb landed** (IMP-006/SL-062); the
   seam is live, not inert.
3. **Cites `IMP-006` as pending → IMP-006 is resolved.** A stale forward-reference.

This is a doc-vs-code conflict on the *storage shape* — exactly the class the
storage rule guards (prose describing tier/shape must track the authored TOML). A
reader trusting SPEC-005 would author ADR supersession the wrong way (expecting a
typed field, not `doctrine link … supersedes` / the `supersede` verb).

## Options
1. **Correct SPEC-005 to current state.** Rewrite `:46-50` and `:81-84`:
   `supersedes` is a tier-1 `[[relation]]` row (SL-095, alongside `related`/SL-048);
   only `superseded_by` + `tags` stay typed (the ADR-004 §5 carve-out); the
   transactional `supersede` verb is **landed** (SL-062), drop the "inert/pending
   IMP-006" framing. Tradeoff: straightforward factual correction; must get the
   carve-out nuance right (superseded_by stays typed *by design*, not by lag).
2. **Correct + add a forward note** that `superseded_by` staying typed is itself
   under review (IMP-032 governance carve-out, IMP-095 record migration). Tradeoff:
   richer, but mixes a factual fix with open-question signalling — cleaner to keep
   the correction pure and let those items stand on their own.
3. **Leave as-is.** Tradeoff: zero effort; but a core governance spec actively
   misdescribes how to author ADR supersession and cites a resolved item as the
   gate — the spec is misleading on its own subject.

## Recommendation
Option 1: a pure factual correction to current state. The supersedes→`[[relation]]`
migration (SL-095) and the supersede verb (SL-062) are both shipped and verifiable
in the ADR TOML and the CLI; SPEC-005 simply lags. Keep the correction tight —
state that `superseded_by` + `tags` remain typed as the deliberate ADR-004 §5
carve-out (not a pending-migration artefact), so the fix doesn't imply more churn
than exists. This pairs naturally with proposal 0002 (SPEC-003 staleness): both are
the earliest specs lagging behind later slices.

Decisions deferred to YOU:
- (a) confirm the **intended end-state** of `superseded_by`: permanently typed (the
  ADR-004 §5 derived-reverse carve-out), or also slated for migration (IMP-032/
  IMP-095 axis)? The correction's wording depends on this.
- (b) **pure correction (1) vs correction-plus-forward-note (2).**
- (c) whether a `spec validate`-style check could ever assert prose/TOML storage-shape
  agreement (hard — prose is free text), or whether this class relies on review.

## Next doctrine move
```
# confirm the drift (read-only):
sed -n '40,50p;80,85p' .doctrine/spec/tech/005/spec-005.md   # the stale prose
sed -n '11,16p' .doctrine/adr/004/adr-004.toml                # supersedes = [[relation]]
doctrine backlog show IMP-006                                  # status: resolved/fixed

# corrective edit is authored-tier — route it (NOT executed; fence forbids):
/route       # → small slice or boot.md-Governance "small backlog item" quick edit
             #   to SPEC-005 §"Identity TOML and the relationships seam" + D2.
```
(Verbs described, NOT executed — fence forbids editing authored spec state.)

## Illustration (optional) — ILLUSTRATIVE, not applied
Hand-authored sketch of the §-correction (no worker):
```diff
--- a/.doctrine/spec/tech/005/spec-005.md
+++ b/.doctrine/spec/tech/005/spec-005.md
-typed `[relationships]` table holding the supersession pair `supersedes` /
-`superseded_by` (the ADR-004 §5 reverse carve-out, ...) and free-text `tags`; the
-`related` axis is a tier-1 `[[relation]]` edge (migrated in SL-048). The
-supersession pair stays typed pending the transactional supersede verb ([[IMP-006]]).
+a `[relationships]` table whose `superseded_by` (the ADR-004 §5 derived-reverse
+carve-out, verb-written) and free-text `tags` stay typed; `supersedes` and `related`
+migrated to tier-1 `[[relation]]` rows (SL-095, SL-048). The transactional
+`supersede` verb (SL-062, [[IMP-006]] resolved) writes `supersedes`/`superseded_by`
+and flips status.
```
