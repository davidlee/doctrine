# Conformance fixture readable-input posture

## Context

The shipped conformance suite proves `BoundedInputSet` — that a capsule's
readable input set is explicit and bounded — inside capsules the fixture itself
builds. `ISS-341`: the fixture builds them by binding **whole host top-level
roots**.

`system_readable_roots` (`crates/doctrine-control/src/conformance.rs`) declares
the *top-level ancestor* of every canonicalized `PATH` entry a readable root,
identity-mapped. On an ordinary NixOS desktop that is `/nix`, plus the host's
entire `/run` and `/var`, because `/run/wrappers/…` and
`/var/lib/flatpak/exports/bin` sit on `PATH`.

`--ro-bind` is read-only, **not `noexec`**. Measured off-jail on a real host:
**691 store paths visible**, and `git`, `curl` (TLS-capable) and `gcc` all
execute inside such a capsule. The junk roots are noise; `/nix` is the finding.

**Production explicitly refuses this shape.** `readable_set`
(`backend/bubblewrap.rs`) binds only declared `readable-roots` plus
`closure-roots` expanded through an operator-declared `closure-resolver`, and
`SL-248` `PHASE-10` `EX-8` states it: *no host-shaped default and no fallback —
no arrangement of configuration can make a host-wide artefact store readable
whole.* The fixture is exactly the host-shaped default `EX-8` rules out, and it
hosts the capsules in which the bounded-input-set property is demonstrated.

No shipped row can convict it. Row 3 reads planted decoys and is silent about
everything else; row 4 derives its permitted-`/` set from `$PATH`, so every such
root is permitted by the very fact that put it there. Both read `Proven`. **The
verdicts are not false — they are worth much less than they read, and nothing in
the transcript says so.**

Found by `ISS-339` (the suite's first off-jail runs). It is the third instance of
one pattern, which is the more general finding: **a derivation that is
accidentally correct in the single environment it has ever run in.** `/bin/sh`
(run 1), the permitted-`/` set (run 2), the root set itself (run 3) — the jail
supplied `/bin` twice over by coincidence and masked each in turn.

## Scope & Objectives

Give the conformance fixture a readable-input posture that is defensible on a
store-managed host, so that what the nineteen rows demonstrate is a capsule
shaped like the ones production builds.

1. **Narrow the fixture's readable set.** Direction from `ISS-341`: have the
   fixture declare `closure-roots` for its shell rather than a top-level
   ancestor — the shape `EX-8` already contemplates.
2. **Resolve the resolver tension.** A closure resolver is host-shaped by nature;
   production escapes that by making it *operator-declared*. A fixture has no
   operator, and must not grow a Nix dependency (`POL-002`). It must therefore be
   closure-correct on a store host **and** still run on a host without a
   resolver — without the fallback quietly reinstating the whole-top-level bind,
   which is the failure mode this slice exists to remove.
3. **Make the posture legible in the transcript.** The present defect is not only
   that the set is wide but that nothing says so. Whatever ships should make an
   over-wide readable set a visible fact rather than an inference.
4. **Dissolve `ISS-340`.** Narrowing the bind set makes `/run` and `/var` stop
   being roots, drops their entries from the inner `PATH`, and collapses row 2's
   coverage set to `/nix` — which is covered. The last red row goes without the
   row being touched and without a criterion-level decision against `PHASE-10`
   `EX-3`.

**Sequencing, and it is not the obvious order: this slice before `ISS-340`.**
Fixing `ISS-340` on its own terms first would weaken `EX-3`'s "from **every**
bound path" *and* leave `ISS-341` standing with its only witness removed. The
cheap fix — filter roots to those bearing an executable — turns the transcript
green while leaving `/nix` bound whole, which is worse than the current red.

## Non-Goals

- **Changing production's readable-set derivation.** `readable_set` is correct
  and is the reference this slice moves the fixture toward.
- **Adding a Nix dependency to the fixture or the crate** (`POL-002`).
- **Re-litigating `EX-8`.** It is the standard being applied, not a question.
- **`ISS-340`'s own terms.** It is dissolved as a consequence, not addressed
  directly.
- **A general host-capability probe.** That is `IMP-417`'s, and the two must not
  be merged: an unavailable backend and an over-wide readable set are different
  facts.

## Summary

To be written at close.

## Follow-Ups

To be written at close.
