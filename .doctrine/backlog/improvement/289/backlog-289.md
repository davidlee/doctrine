# IMP-289: Value-burndown blind to ad-hoc completion

**Source:** SL-219 `/elicit` session, 2026-07-13 (backlog value-refinement).

## Problem

ADR-018 §4's value-burndown post-pass computes an item's *residual* value by
subtracting the raw `value` of any **slice** that `fulfils` it (gated on the
slice being in a delivering state `{started, audit, reconcile, done}`). Residual
is therefore only derivable when completion arrives **through a fulfilling
slice**.

Work that lands **ad-hoc** — a direct commit, a fix folded into an unrelated
slice, a mechanism added without a governing slice — creates no `fulfils` edge.
So burndown never fires, and the engine keeps scoring the item's **full initial
value** indefinitely, until it is manually closed. The residual is invisible.

## Witnessed

- **IMP-255** (dispatch worker full-suite mandate): partly addressed by adding
  `doctrine check` directly (no fulfilling slice). During elicitation its
  authored value read as fully live even though a chunk of the outcome had
  shipped. (In this case follow-up investigation found the residual is in fact
  still ~full — but that was luck, not the model tracking it.) The only edge on
  IMP-255 is `references(concerns) RFC-011`; no `fulfils`.

## Why it matters

Prioritisation by value/effort silently over-weights any item whose work leaks
in outside the slice lifecycle. The more a team direct-lands small fixes, the
more the backlog's value signal drifts high.

## Fix directions (unshaped)

- A way to record partial ad-hoc completion that burns value without a full
  slice — e.g. a lightweight `fulfils`-equivalent edge from a commit/PR, or an
  authored residual-value override distinct from initial value.
- Or: make the value facet explicitly *initial-only* and surface residual as a
  separate, first-class projection that can be fed by non-slice completion.
- Or (cheapest): a lint/report that flags `started`/partly-done items with no
  `fulfils` edge, so a human periodically reconciles their value by hand.

## Related

- ADR-018 (value-burndown, completion-degree facet, fulfils label)
- ADR-016 (derivable-not-relational law that ADR-018 narrowly reverses)
- IMP-255 (the witnessed instance)
- IMP-210 (deferred close-cascade hint — degree feeds it, out of scoring)
