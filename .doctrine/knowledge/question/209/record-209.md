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

## Answer

**Split into three (user, 2026-08-08), landed as REV-049 (*reconcile SL-250*).**

- `REQ-186` **modified** to the hook *set* — every spec's entries, in matcher
  order, across every event it declares, into the *selected* settings file. It
  keeps its identity (the owner-locked merge) and stops naming a file and a
  command form it no longer fixes.
- `REQ-476` (`FR-008`) **introduced** — the `[install] claude-settings-scope` key
  selects the settings file and, through `baked ⟺ gitignored`, the command form.
  One requirement rather than two because they are one decision: splitting them
  would admit an implementation that honours the key and writes a host abspath
  into a committed file, the POL-002 breach the invariant exists to prevent.
- `REQ-477` (`FR-009`) **introduced** — the abandoned-scope sweep: the shared
  ownership predicate, the gate on the target write landing, and the three
  reported outcomes.

**Why not the widened single requirement.** SPEC-011's other seven requirements
are each one sentence over one verifiable behaviour, so a `REQ-186` carrying all
four axes would have been a run-on out of line with its siblings. The
load-bearing objection is narrower: it would have buried the sweep — doctrine's
one destructive write in this area, with its own predicate, gate and reporting
contract — inside a requirement about a merge. Coverage by implication reads as
present and verifies as nothing.

The `T1` half of the question above resolved in favour of the split too, and by
the argument this record already made: behaviour arriving under governance for
the first time is the usual trigger for new requirements rather than a widened
one. RV-350 `F-1` then added a fourth axis the design had not counted (the
command form), which the split absorbs without further widening.
