# IMP-419: Declare a capsule uid no ordinary operator holds

`CAPSULE_UID` is **1000**, and 1000 is the commonest operator uid on a Linux
desktop. Two of row 13's four credential surfaces therefore fail to discriminate
on the design host itself.

## The measurement

`SL-248` `PHASE-10` `VA-1`'s off-jail run (`notes.md` items 131 and 158) reads
four credential surfaces per arm:

| arm | what it shows |
|---|---|
| `P` (positive control, no `bwrap` at all) | `uid=1000 gid=100` — the *same* operator uid as the jail |
| `A1`/`A4` (shipped profile) | `uid_map` `1000 0 1` |
| `A3` (`--uid 4242 --gid 4242`) | moves `uid`, `gid` **and** `uid_map` — `4242 0 1` |

So all four surfaces discriminate **whenever the declared identity differs from
the operator's**. The two dead ones are dead because `CAPSULE_UID` is 1000 and
1000 is what the operator happens to hold — not because the surfaces are inert.
Arm `A3` is the evidence the fix works.

This is not a jail artefact. It bites on any host whose operator uid is 1000,
which includes the design host.

## What to change

Declare a capsule uid (and gid) that no ordinary operator holds, so row 13's
assertion of a *declared* identity is discriminating by construction rather than
by luck of the operator's account. The values are the capsule's own, fixed by the
profile rather than configurable — `sec-5` deliberately carries no credential
knob, and an operator-chosen uid would be a second way to weaken the capsule.

The alternative, considered and rejected at `SL-248`'s reconcile: keep 1000 and
carry the uid half as *stated property that does not discriminate here*. Rejected
because a security assertion that passes for the wrong reason on the most common
host configuration is the vacuous-pass family this slice spent ten phases
removing.

## Caveat

`gid_map` is **unmeasured** on every off-jail arm — the off-jail payload never
reports it (`ISS-338`). So item 131's "only `gid` and `gid_map` discriminate" is
half-verified: `gid` is confirmed off-jail, `gid_map` is not. Settle `ISS-338`
first, or this change lands against a partially-measured baseline.

## Why not in `SL-248`

Net-new work against `PHASE-10`'s closed surface — the same disposition `F-8`
took to `IMP-417`. `EX-11` writes row 13 with all four surfaces regardless, so
nothing in the shipped criteria blocks the change; it is the profile constant
that moves.

Originates from `SL-248` `RV-352` reconcile, `notes.md` items 131 / 158
(phase sheet `F-40`, `T7`).
