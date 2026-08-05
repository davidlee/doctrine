# Predicate staged one phase ahead of its consumer: gate dead_code via cfg_attr(not(test), expect)

A pub(crate) fn used only by tests now (real consumer lands a later phase) needs cfg_attr(not(test), expect(dead_code)) — a bare #[expect] is unfulfilled under cargo test.

## The expect does not travel down the call chain

Staging a helper one *task* ahead of its consumer does not discharge the expect
on what that helper reads. rustc's dead-code analysis walks outward from **live
roots**, so a dead reader does not make its callee live: if `parse()` has no
production caller, the `ALL` table it iterates is still reported dead and keeps
its own `cfg_attr(not(test), expect(dead_code, …))`.

Practical consequence when phasing work: every item in a staged chain carries its
own expect, and they all retire together at the moment the **first production
caller** lands — not incrementally as each link acquires a reader. Predicting
"this expect retires when I add its reader" is wrong unless that reader is itself
reachable from production.

Found at SL-244 PHASE-06 T1 (`Advance::as_str`/`parse`/`to` + `Advance::ALL`),
where the phase sheet predicted the retirement and rustc disagreed.
