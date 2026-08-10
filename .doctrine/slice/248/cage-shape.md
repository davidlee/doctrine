# The cage's shape, and what it does and does not contaminate

Measured 2026-08-10, both sides, by the slice owner. Previously **inferred from a
symptom** (the off-jail audit's § E said so explicitly); now read directly.

Raw readings live in `offjail-audit.md` § E. This file holds what follows from
them, which is the part that changes what anyone does.

## The reading

| namespace | cage | host | shared? |
|---|---|---|---|
| `net` | `4026531833` | `4026531833` | **yes — identical inode** |
| `pid` | `4026535160` | `4026531836` | no |
| `mnt` | `4026535115` | `4026531832` | no |
| `user` | `4026533973` | `4026531837` | no |

Plus: `uid_map 1000 0 1`, `gid_map 100 0 1`, `NoNewPrivs: 1`, `Seccomp: 0`, all
four capability sets zero, 35 `/proc` entries, `setsid` present.

So the cage has **its own user, pid and mount namespaces; shares the host's
network; runs no seccomp filter; has capabilities fully stripped; and already
reads `NoNewPrivs: 1`.**

The `net` row is the one that had to be read on *both* sides to mean anything —
a cage-side inode alone says nothing about sharing. It is the same inode.

## What that splits the remaining rows into

Three groups, and the distinction that matters is **contamination vs coverage**.
A contaminated row measures the cage and attributes it to the capsule. A row with
a coverage gap measures nothing — honest, just absent. They cost differently to
fix, and section E of the audit did not separate them.

### Cleared empirically — `ProcessVisibility`, `ProcessTreeTeardown`

The cage's own pid namespace made these maskable *in principle*, which was the
worry. Both reproduced off-jail **verdict-for-verdict** (`spike-deltas` all four
axes; `spike-teardown-2x2` all six cells). The worry is discharged by
**measurement**, not by argument — which is the stronger discharge, and is why
the audit reported its clean rows rather than only its findings.

### Cleared structurally — `ExplicitNetworkPosture`

A cage that **shares the host's netns** cannot supply the isolation this row
attributes to the capsule. `--unshare-net` on the confining arm really isolates;
a network-permitted weakened arm really reaches the host network. The row
discriminates honestly *in the jail*, and no re-run is owed.

**Residual is a coverage gap, not a contamination one**: no spike exercises this
row at all. Different problem, and a cheaper one.

### Still exposed — `BoundedFilesystemVisibility`, `ImmutableInputSet`

The cage has its **own mount namespace with restricted binds**, so it can supply
exactly the property these two rows attribute to the capsule. **Upgraded from
suspected to confirmed exposure**, and they are now the only confirmed remaining
one.

Nothing in the three existing spikes touches them, so settling them needs a
**new probe**, not a re-run — confining vs weakened arm, run off-jail, readings
handed back for in-jail comparison. In flight with the off-jail session as of
2026-08-10; results land here.

Related and worth checking before that probe runs anywhere uncaged:
`notes_10-12.md:116` records the `va2-write` probe leaving files on `/nix` and
`/bin`. Off-jail those are the **real host paths**.

## `F-38`'s root cause is mechanistic, not situational — and this changes the fix

Recorded because the difference decides which repair is honest.

The measurement was: in the cage, **21 live processes, 0 with `pgrp != sess`**;
off-jail, **759 live, 34 discriminating**. Read as a fact about *that host at
that moment*, the obvious repair is "re-run the group-reader tests somewhere with
a richer process population".

That reading is wrong. **A fresh pid namespace holding ~35 entries, all descended
from a single session, is structurally a population in which nothing can have
`pgrp != sess`.** It is not bad luck. *Any* measurement of process-group
semantics taken inside that cage is non-discriminating, **permanently**.

So a `/proc` reader that slid between the `pgrp` and `sess` field indices can
never be convicted there, and **re-running elsewhere only moves the luck** — it
buys a green that is still a property of where it ran. The fix must be a
**fixture that manufactures a `pgrp != sess` process**, making the test
independent of its environment. That was already the recommendation; the
mechanistic account makes it the only honest one.

This is the same lesson as the vacuous-pass family, one level up: a test whose
discriminating power depends on ambient conditions is not tested by finding
conditions where it happens to discriminate.
