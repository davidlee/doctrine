# ISS-340: Row 2's per-root exec coverage is unsatisfiable on a real host

Row 2 (`Property(BoundedInputSet)`) returns
`Indeterminate { arm: Probe, detail: NoObservation }` on a real NixOS host. It is
the last row standing between the shipped suite and an off-jail admission —
`ISS-339`'s run 3 (2026-08-10) was 18/19 `Proven`, all four claims `Passed`, both
`sec-9` observations `Read` without caveat.

## What the payload requires

Before it tests anything, `execs_only_what_is_bound` establishes a precondition:

```sh
[ "$bound" -gt 0 ] && [ "$ran" -eq "$bound" ] || exit 0
```

Every distinct top-level root among the capsule's inner `PATH` must yield at
least one of `sh cat head env true` that is executable **and runs**. A root that
contributes none prints neither token, which is the `NoObservation` above. That
is deliberate and documented: *"A root that contributed nothing prints neither
token, so a capsule with an unusable input set reads `NoObservation` rather than
a hold that established nothing."*

## Why the premise is false

The payload's own doc comment records the hazard and an escape from it:

> Per `PATH` *entry* would be a different and false requirement: a Nix-style host
> puts one package per entry and most hold none of any fixed candidate list.

The escape is one level too low. Moving from entry to root granularity only
helps where a root has *several* entries beneath it, so that a sparse one is
covered by a sibling. A root whose only entry is sparse simply inherits that
sparseness — there is no sibling to cover it.

Measured on the owner's host, `PATH` entries canonicalized (which is what
`derived_inner_path` does, and what a first diagnostic pass missed — the raw
`PATH` string's first component is *not* the root, because
`/etc/profiles/per-user/…/bin` and `/run/current-system/sw/bin` are symlink farms
resolving into `/nix/store`):

```
COVERED   /nix  <- /nix/store/…-devshell-dir/bin/sh
UNCOVERED /run              /run/wrappers/wrappers.SpYzxokvnp
UNCOVERED /var              /var/lib/flatpak/exports/bin
```

Two roots, each with exactly one real entry, each holding no candidate:
`/run/wrappers/…` is NixOS's setuid wrapper directory (`sudo`, `ping`,
`fusermount`) and `/var/lib/flatpak/exports/bin` is a Flatpak export farm. Both
are ordinary on a desktop NixOS host. Neither is anomalous, and no host
configuration change should be needed to run the suite.

## Why the payload cannot simply be told the answer

`DEC-160` / `EX-16` fix payloads as `Argv` values holding constants — a payload
is not parameterised by the fixture. That is *why* the expectation is
self-derived from `$PATH` inside the capsule. But `$PATH` cannot distinguish the
two reasons a root is bound: `/nix` is bound because the loader and libraries
live there, `/run` only because a `PATH` entry happened to point into it. The
payload sees one undifferentiated list and reasonably assumes every member is an
exec root.

That tension — self-derived expectation versus a bind set with two kinds of
member — is the actual defect, not the shell arithmetic.

## Shapes, and the one worth taking first

1. **Narrow the bind set so the two kinds stop being conflated** — `ISS-341`. If
   the fixture declined to bind a root bearing no executable, `/run` and `/var`
   would not be roots, their entries would leave the inner `PATH`, and `bound`
   would collapse to `/nix`, which is covered. **This dissolves the row without
   touching the row**, and it fixes a second and more serious problem at the same
   time. Prefer it.
2. Weaken to *at least one* root having run. Cheapest, and it keeps the row
   executing rather than statting, but it abandons the "from **every** bound
   path" statement `EX-3` is read against — and it leaves `ISS-341` standing and
   now unwitnessed.
3. Keep per-root coverage but have the payload skip a root it can prove holds no
   candidate. Self-consistent, and vacuous: the skip condition and the coverage
   condition are the same predicate.

Shape 1 is a fixture change; shapes 2 and 3 are criterion-level calls against
`PHASE-10` `EX-3` and belong with the owner, on `RV-352` `F-8`'s precedent.

## Not a fix candidate

Adding `/bin` to the inner `PATH`, or adding a coreutils to the capsule. Both
change what the capsule *is* in order to make a measurement of it pass.

## References

`ISS-339` (the off-jail runs) · `ISS-341` (the shared cause, and shape 1) ·
`IMP-417` (this is another precondition that convicts quietly — see its probe
register) · `PHASE-10` `EX-3` · `DEC-160` / `EX-16` · `RV-352` `F-8` · `SL-248`
