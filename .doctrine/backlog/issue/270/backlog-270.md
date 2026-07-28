# ISS-270: conclude cadence re-arms an unfollowable refresh-base after removing the coord tree

Surfaced at the SL-231 audit (RV-318 F-8).

## Observed

```
$ doctrine dispatch status --slice 231
dispatch: refs/heads/dispatch/231  (988ead5)
coord:    (removed)
trunk:    moved (3 commit(s) ahead of fork-point)
next:     trunk advanced past the prepared base — run 'dispatch refresh-base --slice 231' then re-prepare
```

`git log --oneline dispatch/231..main` is exactly three commits, all written by
the conclude cadence itself: an ISS filing, the `slice status 231 audit`
lifecycle flip, and a case-notes append. Zero code.

## Cause

The documented cadence is refresh-base → verify-vt → `sync --prepare-review` →
**remove the coordination worktree directory** → `slice status <id> audit` →
`/audit`. Steps 4-6 write authored state to the primary tree; those commits land
on `edge`, `edge` is promoted to `main`, and `main` is trunk. So the cadence's
own bookkeeping advances trunk past the fork-point.

`refresh-base` merges trunk into `dispatch/<N>` *in the live coordination
worktree* — which step 4 of the same cadence removed. The prescription is
unfollowable, and because the state never becomes terminal, `dispatch status`
keeps emitting it.

## Impact

Low in consequence, high in confusion. The three commits carry no code, so
nothing about the review surface is stale, and `dispatch candidate create --base
refs/heads/main` folds them in cleanly (verified on SL-231: no conflicts, and
the candidate's `main...HEAD` diff is exactly the 29-file bundle). But
`dispatch status`/`next` is the documented authority for "do the ONE thing it
prescribes", and here it prescribes an impossible action indefinitely — which
either trains the operator to ignore the oracle or sends them to recreate a
coordination tree they just tore down.

## Candidate fix

Either the freshness check ignores trunk movement that is `.doctrine/`-authored
only, or `status` recognises `coord: (removed)` and prescribes candidate-create
instead of refresh-base. The second is more honest: once the coord tree is gone
the slice has left the funnel's altitude, and base freshness is the candidate's
concern.
