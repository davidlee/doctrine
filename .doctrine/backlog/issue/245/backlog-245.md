# ISS-245: dispatch reap patch-id oracle refuses every funnel-managed fork

Surfaced live at SL-228 PHASE-06 — the first phase whose import ran with a
`funnel.toml` row present.

## Symptom

```
dispatch_reap{slice: 228, name: "dispatch/agent-addd7999cd4e54cc9"}
  → MCP error -32603 Internal error            (see ISS-246 for the opacity)

doctrine worktree gc --fork dispatch/agent-addd7999cd4e54cc9 --dry-run
  → dispatch/agent-addd7999cd4e54cc9: not-landed — `--force` to reap, …
```

The fork **was** landed: import returned `Imported{coord_tip: 367d76cd0}`, the
S1 regression diff was green, and the phase concluded with a committed boundary
row.

## Cause (empirically confirmed, not inferred)

`worktree gc`'s landed-oracle is `git cherry` — a **patch-id** comparison, chosen
precisely because the funnel's non-committing 3-way apply severs git ancestry
(`install/dispatch-mechanics.md`, "The import severs ancestry — so 'did it land?'
needs a patch-id oracle").

SL-228 PHASE-05 made the funnel import **atomic**: the worker delta and the
`funnel.toml` `Import` row land in ONE CAS commit (design §3 D1). So the import
commit's patch is a strict superset of the fork commit's:

```
fork   96dca2ddf →  9 files, 1267 insertions(+), 135 deletions(-)
import 367d76cd0 → 10 files, 1274 insertions(+), 137 deletions(-)
                    ^^^^ .doctrine/dispatch/228/funnel.toml | 9 +-

git cherry HEAD dispatch/agent-addd7999cd4e54cc9
  + 96dca2ddf666052b8a31875110fbdcd57350c4d7      # "+" == not upstream
```

Different patch ⇒ different patch-id ⇒ `git cherry` reports the fork commit as
never landed. This holds for **every** funnel-managed fork, not this one.

It did not surface before PHASE-06 because slice 228 carried no `funnel.toml`
until PHASE-06 minted the first row: PHASE-01…05 all imported row-absent, so
their import commits carried the worker delta alone and stayed patch-identical.

## Why it matters

This is a **zero-rescue regression** in the slice whose whole thesis is zero
rescue. `reap` is now unreachable on its own prescribed path: the oracle
prescribes `dispatch_reap`, `dispatch_reap` refuses, and the operator's only
route out is `--force` — the exact "operator learns a `--force` reflex and the
safety gate collapses" failure the mechanics doc names as the reason the
patch-id oracle was chosen over delta-emptiness in the first place.

## Candidate fixes

1. **Ask the funnel, not git.** The record already knows: a phase at position
   `concluded` with `spawn.fork == <branch>` is by definition landed. Make
   `dispatch_reap` consult the funnel record for its landed proof and keep
   `git cherry` only for forks with no funnel row (the legacy path). Cheapest,
   and it is the slice's own idiom — durable committed state over reconstruction.
2. **Compare patches modulo the funnel record.** Diff the import commit with
   `.doctrine/dispatch/<NNN>/funnel.toml` excluded before taking the patch-id —
   the same "modulo the funnel record" trick `conclude_allowed` already uses for
   its stale-tree gate (`src/funnel_machine.rs:741`).
3. Record the fork tip's oid in the `import` row at land time and compare oids
   rather than patches.

(1) is preferred: it reuses state the funnel already commits, and it does not
make the oracle re-derive anything.

## Third facet — the position advance is coupled to gc's success

`worktree gc --force` removes the worktree and branch but lands **no** `Reap`
transition (that is `dispatch_reap`'s half). So an operator who takes the
`--force` route strands the funnel row at `concluded` while the fork it names no
longer exists — the oracle keeps prescribing `dispatch_reap` for a fork that is
already gone.

The recovery is non-obvious and worth pinning: **call `dispatch_reap` anyway**.
`run_gc` treats an absent fork as a clean idempotent no-op, so the tool returns
`{"Reaped": {...}}` and lands the position. Confirmed at PHASE-06:

```
worktree gc --fork … --force   → reaped (worktree/branch as present)
dispatch next --slice 228      → still `reap` (position stuck at concluded)
dispatch_reap{…}               → {"Reaped": …}   # no-op gc, row advances
dispatch next --slice 228      → spawn PHASE-07  ✓
```

That the sequence works is luck of idempotence, not design: nothing tells the
operator to re-issue a verb that just refused. Fix (1) dissolves this facet too.

## Disposition

Not fixed in SL-228 PHASE-06 (out of that phase's exit criteria). The PHASE-06
fork was reaped with `--force` under an evidenced landing, then `dispatch_reap`
was re-issued to advance the row.
