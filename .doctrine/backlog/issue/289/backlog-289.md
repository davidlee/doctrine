# ISS-289: PHASE-15 completed with a VT-2 mandating a test that was never written

SL-233's PHASE-15 is `completed`, but `doctrine slice verify-vt 233` reports:

```text
  PHASE-15:
    ✗ FAIL  VT-2 — pattern `^\s*fn an_edit_inside_materialises_guarded_window_is_never_certified\(`
                   matched no line in `tests/e2e_design_state.rs`
```

## The finding

The mandated function does not exist, and **never did**. `git log -S` over all
refs returns exactly one commit —

```text
063231630 feedback(SL-233): adjudicate RV-324 — six dispositions, DEC-099 supersedes DEC-092, PHASE-15 appended
```

— which is the commit that *authored the criterion*, not one that wrote a test.
So the name has only ever existed as a mandate in `plan.toml`.

Verified with a positive control, because the primary evidence is a negative
grep: `tests/e2e_design_state.rs` is present (83 KB) and carries 61 `fn`
definitions, so the pattern form matches in that file when a match exists.

## Why it matters

The phase was flipped `completed` with an unsatisfied VT criterion. That is the
gap `IDE-008` (executable phase gates at completion-flip) exists to close in
general — this is a concrete instance of the class, found by running the gate
late rather than at the flip.

Two candidate dispositions, and they are genuinely different:

1. **Write the test.** The criterion names a real property — an edit inside the
   materialise-guarded window is never certified — and if that property is
   untested, PHASE-15's evidence is short by one assertion.
2. **Correct the criterion.** If the property is in fact asserted under another
   name (PHASE-13/PHASE-14 VT-1 carry several such round-trip assertions, and
   PHASE-15's own VT-1/VT-3 were WAIVED with exactly that reasoning recorded),
   then the mandate is misspelled rather than unmet, and it should be waived
   with the cross-reference the sibling waivers already use.

Do not guess between them: the sibling criteria in the same phase were waived
with explicit cross-references, so the authoring convention for "asserted
elsewhere" already exists and this one either follows it or is a real gap.

## Provenance

Surfaced while gating VT criteria at the close of SL-233 PHASE-16 T8. Not
caused by that work — the failing file is untouched by it, and the mandate
predates it. Filed rather than fixed in place because the disposition is a
judgement about PHASE-15's evidence, which belongs to SL-233's audit.

Related: [[IDE-008]], [[ISS-271]], [[IMP-235]] — the last two are defects in
`verify-vt`'s *reporting*; this one is a defect in the plan it reports on.
