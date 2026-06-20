# REQ-319: Trunk resolution via freshest_descendant with ancestry close backstop

## Statement

Trunk is resolved by the peeled ladder `DOCTRINE_TRUNK_REF → origin/HEAD → main →
master` folded through `freshest_descendant`: advance only to a strict descendant, so a
stale `origin/HEAD` that is an ancestor of local `main` is overtaken while a
genuinely-diverged candidate keeps ladder order. At the `reconcile → done` lifecycle
crossing, a structural backstop asserts `is_ancestor(planned_new_oid, trunk_tip)` for
the journal's trunk row, fail-closing a slice that projected but was never integrated.

## Rationale

A frozen or first-peel-wins base silently forks work off stale trunk (SL-127). The
ancestry backstop proves *integration occurred* (not tree survival at tip, which other
landed slices may have changed) and stops a slice being marked terminal while its code
sits unintegrated (SL-126).
