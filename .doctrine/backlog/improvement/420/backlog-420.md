# IMP-420: Give a non-refusing capacity verdict a return channel

`CapacityVerdict` has four variants, two of which mean *warn and continue* —
`Low` and `Unknown`. Neither can be returned to a caller.

## Why

`SL-248` `PHASE-06` `EX-6` fixes `provision`'s three parameters and `EX-1` fixes
`CapsuleTransaction`'s nine fields, so there is nowhere for a non-refusing
outcome to go. What ships: `provision` refuses when the verdict refuses, and
otherwise writes **one structured `key=value` line to a locked stderr**.

That is a side effect in a function the design describes as returning a value,
and it is invisible to a non-terminal caller — a library consumer gets no
programmatic access to a `Low` verdict at all. On a host that is quietly filling
up, the only signal is a line on a stream nobody is required to read.

`SL-248` reconcile named the stderr channel explicitly in `design.md` § *Capacity*
rather than leave a reader to discover it, but naming it is not fixing it.

## Options

1. **An observation field on the transaction.** Fits the existing shape — the
   caller already receives a `CapsuleTransaction`. Costs an `EX-1` field.
2. **A sink parameter.** Explicit, testable without capturing stderr, and keeps
   the verdict out of the transaction's identity. Costs an `EX-6` parameter.
3. **Keep stderr, documented.** What ships today.

Both (1) and (2) change a criterion `SL-248` closed over, which is why neither
was taken there.

Originates from `SL-248` `RV-352` reconcile, `notes.md` item 45
(phase sheet `PHASE-06` `F-6`).
