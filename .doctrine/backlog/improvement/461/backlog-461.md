# IMP-461: Slice scope has no staleness signal against its design

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`RV-370`'s rounds 2, 3, 4 and 5 each integrated a design repair into
`design.md` and each left `slice-246.md` asserting something the repair had
falsified. Round 4's `F-22` caught two stale counts in it, round 4's `F-26` four
stale `file:line` ranges, round 5's `F-28` a signature the design had just
changed in three places. Four consecutive rounds, three separate classes of
staleness, one artefact.

## Why it recurs

`design.md` is section-fingerprinted by the design run: a hand edit that moves a
section changes its digest, `review_pass` goes `STALE`, and the section's
attestation is void by construction. That is a machine noticing.

The slice scope has no equivalent. It carries no run revision, no per-section
digest, and no relation to the design's revision — so nothing makes it stale when
the design moves under it. It is reconciled by whoever remembers, which over four
rounds was three times nobody.

This is not `SL-246`'s defect. It is a property of how the two artefacts relate,
and it will recur on the next slice whose design is repaired more than once.

## Worth considering

Not a design, just the shape of the options:

- a scope↔design coherence check at `design materialise` or in `doctrine
  doctor` — cheap, but needs something checkable, and the scope is prose;
- fingerprinting the scope's claim-bearing sections the way the design's are,
  so a moved design section marks the scope outstanding;
- or accepting it and making the reconciliation a named beat of the `/feedback`
  integration step, which is where it keeps being missed.

The third is the cheapest and the weakest; the first two are the ones that make
the machine notice rather than the agent.

## Provenance

`RV-370` `F-28` (raised round 5, against `SL-246`), with `F-22` and `F-26` as
the prior instances. The instance was repaired in `slice-246.md`; this item
carries the pattern.
