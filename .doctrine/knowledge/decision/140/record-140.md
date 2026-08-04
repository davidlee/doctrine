# DEC-140: Verification evidence lives at the tier that can produce its subject

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The decision

A `VT` row's `test_file` and its `expects` must name a tier that can actually
produce the row's subject. Where a clause presumes a reader that a phase does not
own, the clause moves to the phase that owns the reader — it is not asserted
hollowly against a type shape in the declared file to turn the row green.

Corollary: when the owning phase already carries the clause, drop it rather than
restate it. Two rows asserting one fact is the duplication this project treats as
a defect everywhere else.

## What forced it

`SL-244` `PHASE-04` `VT-4` originally read *"an RV the shell cannot read produces
no `ObservedReview`, **and the gate reads that as unmet rather than as
satisfied**"*, with `test_file = src/design_run/tests.rs`.

The second clause has no subject in that phase. The condition that reads
`ObservedReview` is `review-disposition-attested`, and it lands in `PHASE-05`
along with the `ReviewDisposition` act that names a review at all — verified
against the full `Condition` vocabulary (`src/design_run/gate.rs`, ten members,
no such row) and against `PHASE-04`'s own exit criteria, which contain no
gate-row criterion. So nothing in `design_run` could read the field, and a test
in the declared file could only have asserted that `Option` has a `None`.

The first clause is real, but its subject — an unreadable ref — exists only in
the shell. The declared `test_file` pointed away from the only tier that could
produce it.

`PHASE-05`'s `VT-9` already carried the dropped clause verbatim: *a stored act
naming an unreadable RV is `ReviewUnavailable` and names no findings*. The row
was prospectively duplicating a row that already existed.

## What was done

`VT-4` amended in place (ids are immutable; the text appends, dated and reasoned
inline): the gate clause dropped as `PHASE-05` `VT-9`'s, `test_file` moved to
`src/review.rs`. Recorded in `plan.md` § *One dependency the plan flags rather
than re-decides*.

## The generalisable part

The failure mode is a criterion authored top-down against the finished design
rather than against the phase boundary it lands on — the clause is true of the
system and false of the phase. It reads as satisfiable right up to the moment
someone looks for the reader, and the cheap resolution (a shape assertion in the
declared file) is indistinguishable from the real one on a `verify-vt` summary.

Detect it at `/phase-plan`: for each `VT`, name the reader that observes the
asserted effect and check the phase owns it. `SL-244` `PHASE-04`'s sheet caught
this one as `R1` at plan time and it still needed a consult to settle, which is
the honest cost — planning surfaced it, it did not resolve it.

Related: [[mem.fact.design-run.e2e-counts-embed-the-unit-suite]] is the other
`verify-vt` reading trap on this slice — a derived count read as a control.
