# QUE-211: Backend axis: many mechanisms under one contract, or assurance tiers?

## The question

`SPEC-030` `REQ-459` states backend admission as **property-equivalence**: every
backend enforces equivalent freshness, filesystem, process, credential, network,
resource and canonical-authority properties, and `Admission` has *exactly one
green path* — `Admitted` iff every row in tables A and B is `Proven`.

That works when backends differ only in **mechanism**. The owner's stated intent
for the capsule programme is that doctrine ultimately wants **two shapes**:

- **shape A** — runs on macOS without large investment, for casual adopters;
  plausibly Seatbelt as a bubblewrap surrogate;
- **shape B** — a higher-security shape for a dedicated server or NixOS, which
  is what the Firecracker/microvm spike promises.

Shape A is not a different mechanism delivering the same assurance. It is
explicitly *cheaper and weaker*. So: **is the backend axis mechanism, or
assurance tier?**

## Why it is not decorative

Under today's type, a shape-A capsule and a shape-B capsule that each pass their
rows both print `Admitted`. Nothing in the outcome distinguishes the boundary the
reader is looking at.

That is the `ISS-341` pattern — *"the verdicts are not false — they are worth
much less than they read, and nothing in the transcript says so"* — at the level
of the verdict type rather than the fixture. Third instance of the family, and
the first that is structural rather than a derivation accident.

## Candidate answers

**A — one contract, many mechanisms (status quo).** `REQ-459` unchanged.
Mechanisms that cannot meet the property set are simply not admitted; there is no
second, lower bar. Consequence: shape A ships only if Seatbelt genuinely meets
the same properties, or it does not ship.

**B — tiers with distinct contracts.** The property set is tier-indexed, the
tier rides the verdict, and a capsule is admitted *against a named tier*. `REQ-459`
needs a `REV`. Consequence: the difference is visible in the artefact rather than
inferable from the backend id.

## The honest counter-argument for A

`AdmissionVerdict` already carries `backend: BackendId`, `host: HostDescriptor`
and `date`, precisely so criterion 3's *independently* has an artefact to point
at. A reader who knows the mechanism can therefore already infer the assurance.

Whether inference is sufficient is the crux. The `ISS-341` family says no —
"worth much less than it reads" *is* the gap between what a verdict states and
what a reader must infer. But that is an argument, not a settled rule, and it is
the argument this record exists to have.

A third possibility worth naming: shape A is admitted under a **different
property set** rather than a weaker verdict on the same one, in which case the
tier is carried by which rows exist rather than by a new field — which is
`DEC-189`'s mechanism, reused.

## What it gates

`DEC-190`'s verdict-kernel extraction. The kernel is a few hundred lines of
policy and it is the part that must not be redone; a tier-blind kernel is a
kernel rewritten. **Settle this before the extraction, not after.**

It does not gate the `Fork 3` lifecycle spike, which measures whether a
doctrine-driven microVM can be fresh-per-transaction at tolerable cost and is
indifferent to how the verdict is typed.

## Related

`RSK-231` (why the programme is being revisited), `IMP-405` (SPEC-030 keys
backends to platforms; the wording fix that makes a second Linux mechanism legible
as a peer), `DEC-189`, `DEC-156` (a profile mints its own `BackendId`; the
contract was built to bind mechanisms nobody has written yet).


---

## Answered, 2026-08-11 — by `DEC-191`, and neither candidate was taken

The owner refused **A** (one contract, many mechanisms) *and* **B** (tiers).
Part one of the ruling kills equivalence; part two kills ranking:

> Seatbelt for example is stronger than bubblewrap on some fronts, weaker on
> others, and the complements available on linux vs macos vary greatly in their
> ability to make up the rest. Better to honestly account for the differences
> than to try to prove equivalence.

A tier is a scalar. Assurance is not ordered, so a tier forces the same false
comparison equivalence does, one notch quieter. `DEC-191` takes a **vector over
fronts**: authority is an invariant floor, confinement strength is a published
per-front profile, and the verdict stops collapsing a structure it already
carries.

The third possibility this record named in passing — *the profile is carried by
which rows exist rather than by a new field* — is the one that survived, via
`DEC-189`.