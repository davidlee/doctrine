# QUE-213: What bounds a host-initiated capsule fetch?

Raised 2026-08-11 by `DEC-192` residual 1, which the decision deliberately did
not close.

## The question

`git fetch` has no `receive.maxInputSize` analogue. That knob is *receive*-side,
and `DEC-192` removes the receive side from the host entirely. So the host runs
`index-pack` over guest-authored bytes with **no configured ceiling at all**, and
whatever bounds it has to come from outside git.

This is not a new risk — every candidate transport, bundles included, parses a
hostile pack host-side, and `EVD-006` found the hazards identical on fetch and
bundle. What changed is that a knob existed and now does not.

## Candidates

* `systemd-run --user --property=MemoryMax=… --property=RuntimeMaxSec=…` (or
  plain `ulimit`) around the fetch — bounds the *damage*, not the input.
* `--depth` / shallow fetch — bounds the *input*, and changes what arrives.
* A byte ceiling enforced by the transport wrapper rather than by git.
* `transfer.fsckObjects = true` on the quarantine repo — orthogonal (validity,
  not size), wanted regardless, and unmeasured for cost (`EVD-016`).

## What must not be used as the bound

`target.sizes.volume` — 32 GiB in the spike. It is a disk size, not an admission
bound; treating it as one grants the guest 32 GiB of host `index-pack` work per
fetch. `IMPORT.md` named this trap and it should not be re-walked.

## Why it needs deciding rather than drifting

`REQ-451`'s six bounds — path, symlink, quiescence, time, byte, object — were
written for a file-appearing-in-a-shared-location hazard that this architecture
does not have. Path, symlink and quiescence lose their subject under `DEC-192`.
**Byte and object survive the transport change and land here.** Shaping
`REQ-451`/`REQ-452` (`IMP-426` P1d) needs this answer, so it blocks the same item
`QUE-212` just stopped blocking.

## Related

`DEC-192` (raised it), `EVD-016` (the measurement that did not include it),
`REQ-451` / `REQ-452` (what it feeds), `DEC-133` (the sibling residual —
retention of a quarantine repo as the only candidate exhibit), `IMP-426`.


## Partially answered 2026-08-11 — and `ulimit -f` is not the whole bound

The spike's `capsule-collect` runs the fetch under a `ulimit -f` ceiling sourced
from `target.nix`. That is a real bound and worth keeping, but it does not close
this question, because `RLIMIT_FSIZE` caps **one file's size**, not a transfer:

* a pack of very many small objects never trips it and fills the disk;
* a delta bomb never trips it and is paid in `index-pack` memory and CPU, not in
  written bytes.

So the honest status is: the *packfile* is bounded, the *transfer* is not. What
would close it, in rough order of value — a filesystem quota or dedicated volume
for the quarantine directory (bounds disk properly), `MemoryMax`/`ulimit -v`
around the fetch (bounds the expansion), and `--depth` (bounds the input).
`transfer.fsckObjects=true` is already set and remains orthogonal: validity, not
size.
