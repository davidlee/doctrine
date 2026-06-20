# REQ-311: Ref taxonomy and two mutability classes

## Statement

Every branch the system creates is classified into exactly one of two classes, and
the class governs its lifecycle. **Mutable** refs are advanced in the normal course of
a run: `dispatch/<N>` (the coordination SSoT, funnel's sole write target),
`candidate/<N>/<label>` (interaction branch; tip mutable, recorded OIDs immutable),
`edge` (optional standing aggregate), and trunk (`main`/`master`/`origin/HEAD`,
advanced only by `integrate --trunk`). **Immutable evidence** refs (R2) are
`review/<N>` (impl bundle) and `phase/<N>-NN` (per-phase code cut). No verb mutates an
evidence ref in place.

## Rationale

The mutability class is the load-bearing fact about each ref: it determines whether a
ref can be trusted as a fixed audit input or must be treated as moving. Writing the
taxonomy down removes the ambiguity that let an external reviewer treat an evidence ref
as an ordinary review branch (the SL-067 trap).
