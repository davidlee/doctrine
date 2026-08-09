## The shape

`SL-249` `PHASE-07` amended two governance entities from a four-kind record set
to seven, and shipped `tests/governance_kind_coverage.rs` as a standing canary so
the next kind cannot land without both entities moving. The canary is good: 11
tests, and against the pre-amendment corpus it reports exactly 23 failures, so it
is not vacuous.

The same amendment left a defect the canary **cannot** see. Adding the `evidence`
facet introduced a fourth closed enum (`provenance`), and two later sentences
enumerating the closed enums still said three. A stale *facet-enum* list carries
no kind name, no prefix and no numeral — so nothing the checker measures moves.
The phase's own guard could not have caught the phase's own residue.

It was found by `VA-1`: an agent read of the amendment, asking a different
question with a different reader.

## Why this matters beyond the instance

The tempting inference is "widen the canary". That is usually wrong. A test
encodes *one* invariant precisely; the residue of a change tends to land in the
adjacent shapes the invariant does not name, and there is no finite widening that
closes the class. The right inference is that a `VA` criterion is **not**
redundant with a `VT` criterion over the same artefact — they fail differently,
and a phase that ships only one of them over a prose amendment is under-verified.

## How to apply

- When a phase both *makes* a change and *ships a guard against that class of
  change*, do not treat the guard as covering the change. Ask what the change
  touched that the guard does not measure — and put a differently-shaped reader
  on it.
- Prefer a `VA` (agent-mode) criterion alongside the `VT` whenever the artefact
  under change is prose. Prose has no type system; the canary only sees the parts
  someone thought to encode.
- State the blindness explicitly at close. "The canary does not cover X" is
  cheap to write while the reasoning is live and expensive to reconstruct later.

Related: [[mem.pattern.verification.re-derive-every-inventory-at-use]] — the
`R-inventory` shape this defect also belongs to.
