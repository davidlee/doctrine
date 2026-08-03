A batch of findings is not a flat list. Some findings' resolutions **relocate or
delete another finding's fix site**, and some **change another's economics**. Rule
in dependency order or you will write a correct fix against a design the next
ruling moves.

Two failure shapes, both observed in one `SL-244` self-review batch of nine:

- **The fix site moves.** Two findings touched the same row. Splitting a coverage
  variant (finding A) plausibly relocated where a second finding's (B) lane
  requirement belonged — B's fix might have landed on a slot A was about to
  delete. Ruling A first settled where B's fix goes. Had B gone first, its fix
  would have been rewritten by A.
- **The economics change.** A finding recommending a new closed sub-enum was
  argued from "make the illegal value unrepresentable". A *later* finding in the
  same batch generalised an admission check over the whole record — which gave
  the same guarantee at zero new type cost, and reversed the first
  recommendation. The reversal was only visible because both were on the table
  together.

So, before disposing any of them:

1. Group findings that **mutate the same declaration**. They are one edit, not N.
   Integrating them separately rewrites the same block N times, which is how a
   previous integration introduces the defects the next review finds.
2. Ask of each pair: *does A's fix change where B's fix goes, or whether B is
   still worth doing?* Rule the constrainer first.
3. Keep the reversal cheap. Say plainly when a later ruling has invalidated a
   recommendation you already made — see
   [[mem.pattern.review.observation-right-prescription-wrong]].

The corollary for the finder: **present the dependency graph with the batch**, not
just the ranked list. Severity ordering and dependency ordering are different
orders, and the second is the one that governs integration.

Related: [[mem.pattern.review.observation-right-prescription-wrong]],
[[mem.pattern.design.classify-at-authoring-not-from-behaviour]].
