# ISS-449: verify-vt claims keyword present on an unattributable row it never checked

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

`doctrine slice verify-vt 256`, run mid-`PHASE-01`, printed the same verdict for
all twelve VT rows across the slice's three phases:

```
  PHASE-02:
    ≈ UNATTRIBUTABLE VT-4 — keyword present but `src/design_run/change_log.rs` not modified by this slice
```

For `PHASE-01` the keywords genuinely are present. For `PHASE-02` `VT-4` they
are not — that phase has not been implemented. Its mandate names
`const READABLE`, `const EMITTABLE` and `const fn is_subset`, and each greps to
**0** in `src/design_run/change_log.rs`. Positive control: `const ALL`, which
`PHASE-02` will delete, greps to 1. So the clause *"keyword present"* is
boilerplate in the `UNATTRIBUTABLE` arm, not a fact the verb established.

## Why it matters

`verify-vt` is evidence. A reader at audit — or an agent deciding whether a
phase's verification is wired — takes *"keyword present but not modified by this
slice"* to mean **the mandate is satisfied and only the attribution is missing**.
For an unimplemented phase that is exactly backwards: nothing is satisfied, and
the row is silently reported as though the only gap were bookkeeping.

It is adjacent to `STD-003` (*No silent skip — a degraded read is disclosed*)
and arguably worse than what that standard forbids: the verb does not skip
quietly, it **affirms the thing it did not check**.

## What is fine

The `UNATTRIBUTABLE` state itself is benign and expected. The slice's
arm-neutral source-delta registry has no row until a phase flips to `completed`,
so mid-phase there is nothing to attribute against. `doctrine slice conformance`
reports the same underlying condition honestly:
`conformance unavailable — no recorded source deltas`.

## Confirmed by before/after on the same tree

`PHASE-01` was then flipped to `completed`, which records the phase's source
delta and makes the rows attributable. Re-running the identical command on the
identical source turned the boilerplate rows honest:

```
  PHASE-02:
    ✗ FAIL        VT-4 — keyword `const READABLE` absent from `src/design_run/change_log.rs`
```

Same file, same keyword, no source change between the two runs — only the
attribution. So the verb *can* check the mandate and does, on the attributable
path; the `UNATTRIBUTABLE` arm simply returns before checking and prints the
claim anyway. `PHASE-03` `VT-2` remained `UNATTRIBUTABLE` in the second run
(`snapshot.rs` is not in the phase's delta) and still asserted *"keyword
present"* for a keyword that is absent — a third instance.

## Suggested shape of the fix

Either evaluate the keyword mandate before short-circuiting on attributability
and report the two facts independently — e.g.
`UNATTRIBUTABLE VT-4 — keywords 0/3 present; <file> not modified by this slice`
— or drop the unchecked clause entirely and say only what was determined. The
first is more useful mid-phase; the second is the minimum honest change.

## Provenance

Found while executing `SL-256` `PHASE-01`, running `verify-vt` to discharge that
phase's `VT-4`. Not a blocker there: `PHASE-01`'s four mandates were confirmed
present by direct grep with positive controls, and the attribution resolves once
the phase flips to `completed` and its source delta is recorded.
