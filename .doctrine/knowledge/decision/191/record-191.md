# DEC-191: Confinement assurance is a per-front posture, not a rank

Answers `QUE-211`. Owner's ruling, 2026-08-11, in two parts.

**Part one — equivalence is refused:**

> I'm pretty sure a bubblewrap-based solution cannot possibly be made as secure
> as a decent microVM backed one can. And it is a fool's errand to pretend
> different backends have identical security postures — they just don't.

**Part two — and so is ranking:**

> Seatbelt for example is stronger than bubblewrap on some fronts, weaker on
> others, and the complements available on linux vs macos vary greatly in their
> ability to make up the rest. Better to honestly account for the differences
> than to try to prove equivalence.

`QUE-211` offered "one contract, many mechanisms" against "tiers with distinct
contracts". Part two refuses **both**: a tier is a scalar, and assurance is not
ordered. A rank would force exactly the false comparison equivalence forces, one
notch quieter.

## The decision

Confinement assurance is recorded as a **vector over fronts**, and the verdict
publishes the vector.

- A **front** is an axis on which a boundary can be stronger or weaker — kernel
  attack surface, filesystem reach under compromise, escape consequence, network
  perimeter, resource ceilings, credential reach, process containment.
- A **backend** is a *composition* of primitives, not one primitive. bubblewrap
  plus Landlock plus seccomp plus cgroups is a different composition from
  Seatbelt plus hardened runtime, and the two compositions have different
  per-front profiles because the available complements differ by platform.
- The verdict states **what was established on each front, by whichever
  composition established it**. It does not reduce that to a word, a score, or a
  position in an ordering.

**Every front named here is an *escape* front, and the profile must say so**
(`CPT-002`). It is an honest account of the wall and is silent about the work
done inside it — where the headline threat actually lives. A reader must not take
a strong confinement profile as a strong safety claim, and a strong profile
licenses *more* weight on trusted-side inspection rather than less: as escape
gets less likely, inspection carries a larger share of the residual risk.

## Authority is the one axis that is not a front

`REQ-459` today enumerates one undifferentiated list — *"equivalent freshness,
filesystem, process, credential, network, resource, and **canonical-authority**
properties."* Canonical-authority does not belong in that list with the others.

What a capsule worker may *do* — no canonical repository or ref mutation, no
canonical credentials, no writable shared Git object store, no control-plane
state; local edits and local commits only — is the transaction boundary
`ADR-020` selected. It is invariant across every backend, without exception. A
composition that weakens it is not a weaker posture; it is not a capsule.

So the split is: **authority is a floor, assurance is a profile.** That is what
lets this decision land without reopening `ADR-020`, whose step 2 — *"a platform
backend that enforces the same authority properties"* — sits entirely on the
floor.

## Why this is cheaper than it sounds

`AdmissionVerdict` already carries `rows: Vec<(RowId, RowVerdict)>` — documented
as *"every row of tables A and B, **including the proven ones**"* — plus
`backend`, `host`, `date`, table C claims, and unrowed observations. **The vector
is already in the type.**

What is wrong is only the final reduction: `Admission::Admitted` is an
all-or-nothing AND over a fixed row set, so a structure that already records
per-front detail is collapsed to one word before a reader sees it. The fix is to
**stop collapsing**, not to add a ranking apparatus.

Combined with `DEC-189` — row membership is a function of the mechanism's
available deltas — the shape falls out: each composition's suite has the rows its
fronts admit, each row's verdict stands on its own, and the artefact is the set,
not a summary of it.

## What follows

1. **`ADR-020` stands unamended.** Verified against its text, not assumed.
2. **`REQ-459` needs a `REV`.** Its property list splits along authority-floor
   versus assurance-profile, and its single green path goes. Present-tense
   governance routes through a Revision (`ADR-013`), not a direct edit;
   `IMP-405`'s rename folds into the same revision rather than landing
   separately.
3. **`DEC-190`'s kernel is specified, not merely unblocked.** It is
   front-structured from the first line — the thing that must not be retrofitted.
4. **A backend does not self-declare its profile.** It is established by passing
   the rows its fronts admit. Otherwise the profile is a label, and labels are
   free.

## What is deliberately not decided here

- **The front list.** The seven named above are a starting sketch, not a closed
  enumeration, and closing it is real design work. `CPT-002` bounds what closing
  it can achieve: the list is about escape, and no amount of completeness on it
  addresses heresy.
- **The admission floor.** "Not ranked" must not become "nothing can fail". Some
  fronts are presumably mandatory at some strength for *any* admitted backend,
  and which ones is unset. A profile nothing can fail is `ISS-341`'s defect
  family one more level up — the fourth instance, if it happens.
- **How a reader compares two profiles** when choosing a backend. Refusing a
  total order does not remove the operator's need to choose, and a wall of
  per-front detail can obscure as effectively as a single word.
- **Whether this wants an ADR of its own.** Project-global with architectural
  consequences is the `ADR` test and this passes it; it is also expressible as a
  `REV` against `SPEC-030` under `ADR-020`'s existing authority. Owner's call.

## Related

`QUE-211` (answered by this), `DEC-189` (row membership is per-mechanism — the
mechanism by which two profiles legitimately differ), `DEC-190`, `EVD-015`,
`RSK-231`, `IMP-405`, `ADR-020`, `SPEC-030` `REQ-459`.
