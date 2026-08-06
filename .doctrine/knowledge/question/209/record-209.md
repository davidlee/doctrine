# QUE-209: REV requirement granularity for the new hook set

## The question

SL-250's REV amends SPEC-011 ([[DEC-171]]). Does it **widen `REQ-186`** to cover
the whole new hook set and the scope key, or does SPEC-011 also gain **new
requirements** alongside an amended `REQ-186`?

## Why it is open

`REQ-186` today reads:

> `boot install` merges a `<exec> boot` SessionStart hook into Claude
> settings.local.json, refreshing a stale owned copy and preserving every foreign
> hook and key.

SL-250 invalidates it on three axes at once — six `HookSpec`s across five events
([[DEC-162]]), a scope-selected project default remembered in `doctrine.toml`
([[DEC-163]]), and the abandoned-scope sweep ([[DEC-164]]). One requirement
carrying all three may be under-modelling.

The sharper half: SL-250 triage finding `T1` established that four of the
commands being wired — `worktree nominate`, `worktree denominate`,
`worktree pretooluse` and `memory surface` — have **no spec and no ownership
predicate at all** today. They exist only as plugin JSON. So this slice does not
merely relocate specified behaviour; it brings previously unspecified behaviour
under governance for the first time, which is the usual trigger for new
requirements rather than a widened one.

## Disposition

**Deferred to reconciliation (user, 2026-08-06.)** Raised during SL-250's design
run `dr-019fd692` at the sufficiency gate and explicitly deferred rather than
added to the inquiry map: the REV is authored at reconcile, and requirement
granularity is the natural call at the point of authoring rather than a design
decision made in advance of it.

Not a blocker for drafting, and recorded here so reconciliation inherits it
instead of rediscovering it.
