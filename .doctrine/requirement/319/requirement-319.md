# REQ-319: Trunk resolution via freshest_descendant with ancestry close backstop

## Statement

Trunk resolution has two cases:

1. **Explicit override.** If `DOCTRINE_TRUNK_REF` is set, it must resolve to a commit and
   **wins outright** — it is not compared against or folded with the fallback refs, and an
   unresolvable value is a hard error.
2. **Fallback ladder.** Otherwise resolve `origin/HEAD`, `main`, `master` (first-seen,
   de-duplicated) and fold those through `freshest_descendant`: advance only to a strict
   descendant, so a stale `origin/HEAD` that is an ancestor of local `main` is overtaken
   while a genuinely-diverged candidate keeps ladder order.

Separately, at the `reconcile → done` lifecycle crossing a structural backstop asserts
`is_ancestor(planned_new_oid, trunk_tip)` for the journal's trunk row, fail-closing a
slice that projected but was never integrated.

## Rationale

A frozen or first-peel-wins base silently forks work off stale trunk (SL-127). The
ancestry backstop proves *integration occurred* (not tree survival at tip, which other
landed slices may have changed) and stops a slice being marked terminal while its code
sits unintegrated (SL-126).
