# The verdict kernel classifies as a leaf

`SL-253` `inq-4` asked where `BackendId` lives after the split, framing it as a
possible new `ADR-001` edge: `BackendId` sits in `backend.rs`, which the layering
map classifies **leaf**, while `conformance` is **engine**.

## The framing does not survive the map

`ADR-001`'s classification rule is *engine = imports engine + leaf*. Engine to
leaf is the permitted direction, and `conformance`'s own row in `layering.toml`
already declares `→ provision, transaction, backend, config, host`. The edge
exists, it is legal, and nothing new is created by the kernel using it.

## The question worth asking instead

What tier is the kernel *itself*? Under `DEC-196` the kernel is the taxonomy,
`RowVerdict`, the verdict types, `row_verdict` and `verify_over`. Its imports are
std, `backend` (leaf) and `host` (leaf). Nothing reaches `provision` or
`transaction`.

So the kernel **classifies as a leaf** — and that is the prize. `DEC-190`'s
central claim is that the kernel is reviewable without loading a confinement
mechanism. A tier is the one form of that claim a machine can check, so it is
made a required exit criterion rather than left as a happy consequence: a later
phase that reaches for `provision` fails the architecture gate instead of quietly
re-entangling the file.

## A third inlet DEC-196 did not enumerate

`ConformanceBackend` (`conformance.rs:630`) is not kernel-tier under the seam
`DEC-196` cut. Every method past `as_capsule_backend` takes a `PropertyRemoval`,
an `AuthorityGrant` or an `Under` — all three placed payload-side. So
`verify_over`'s `&dyn ConformanceBackend` parameter points kernel to payload,
which neither `inq-3` nor `DEC-196` named.

It closes in `DEC-196`'s own direction. `verify_over` calls exactly two methods
on the backend — `id()` and `availability()` — and both belong to
`CapsuleBackend`, on the leaf side. Everything else it does with the backend is
hand it to the row runner, and a payload-supplied runner keyed on `RowId`
captures its own concrete backend. So the parameter narrows to
`&dyn CapsuleBackend`, or drops.

That narrowing is what makes the leaf classification *hold* rather than merely
look plausible. `DEC-196` stands unamended; this is a refinement in its
direction, recorded here.

## What stays put

`BackendId` does not move. Its own doc comment states why it belongs to the
backend contract: a `&'static str` newtype rather than a closed enum,
deliberately open so a profile nobody has written yet mints its own constant
(`DEC-156`). That is the contract's business, not the verdict's. `DEC-190`
naming it as kernel content reads as *the kernel depends on it and must not close
it*, which the existing placement already satisfies.
