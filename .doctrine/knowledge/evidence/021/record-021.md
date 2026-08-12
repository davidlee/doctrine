# EVD-021: Vacuous admission is reachable and its guard is per-suite

Found while verifying `SL-253`'s pre-design research, 2026-08-12.

## What was checked

`crates/doctrine-control/src/conformance.rs:5166`:

```rust
fn admission(rows: &[(RowId, RowVerdict)]) -> Admission {
    if rows.iter().all(|(_, verdict)| matches!(*verdict, RowVerdict::Proven)) {
        Admission::Admitted
    } else {
        Admission::NotAdmitted { reason: NotAdmitted::Rows }
    }
}
```

`.all()` over an empty slice is vacuously true, so an empty row list is
`Admitted`. This is not a discovery — the suite knows. `conformance.rs:11275`:

```rust
fn an_empty_row_list_is_admitted_and_the_shipped_tables_are_what_prevent_it() {
    assert_eq!(admission(&[]), Admission::Admitted);
    assert!(!tables().is_empty(),
        "the shipped tables are what stop the vacuous path being reachable");
}
```

Its doc comment says the repair was declined on purpose: `T11` was an audit and
`admission`'s body was `T13`'s territory, so *"a drive-by hardening would have
been the phase editing the thing it was auditing."* That reasoning was sound at
the time and is not being second-guessed here.

## Why it matters now, and it is a change of circumstance rather than a defect

The guard is **external to the reducer** and it guards **one fixed table set**.
`tables()` is non-empty, therefore the vacuous path is unreachable — true today,
and true because exactly one suite exists.

`DEC-189` makes row membership *a function of the mechanism's available deltas*.
Four of the fourteen rows lose their host-facing delta under a hypervisor
boundary; a microVM earns rows bubblewrap never could. Once membership is
per-mechanism, "the shipped tables are non-empty" is no longer a statement about
**the** table set — it is a statement about whichever set a given backend
happens to bring. A mechanism that contributes few rows, or none, is admitted on
the strength of an assertion made about a different backend's tables.

`DEC-191` removes the other half of the protection in the same move: it stops
the AND-reduction, and its own § *What is deliberately not decided here* names
the exposure — *"'Not ranked' must not become 'nothing can fail'. Some fronts
are presumably mandatory at some strength for any admitted backend, and which
ones is unset."*

So two accepted decisions jointly dissolve the only guard on a path the suite
already documents as reachable.

## What follows

`SL-253`'s `OQ-2` — where the admission floor sits — is promoted from an open
question to a **blocking design decision**, to be settled before the extraction
rather than after it. The floor cannot be inherited from the reducer, because
the reducer is what is being removed, and it cannot be inherited from
`tables()`, because `DEC-189` makes that per-mechanism.

This is `ISS-341`'s defect family — *"the verdicts are not false; they are worth
much less than they read, and nothing in the transcript says so"* — one level up
from the readable set, and arriving through the reducer rather than the fixture.
`DEC-191` predicted the fourth instance; this is the mechanism by which it would
happen.

## Limits

- Verified by inspection at working tree `94d0b5603`; not by running the suite.
- The claim is about **reachability of the code path**, not that any backend
  today produces an empty or short row list. Bubblewrap does not.
- Whether the floor is a row-count minimum, a set of mandatory fronts, or
  something else is not decided here — only that it must be decided.

Related: `DEC-189`, `DEC-190`, `DEC-191`, `ISS-341`, `SL-253` `OQ-2`.
