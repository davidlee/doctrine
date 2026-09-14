# IMP-446: Split Declaration into wire and stored types

`Declaration` serves two roles at once. It is the **wire type** — one entry in a
submission's `declare: [...]` (`src/design_run/submission.rs:122`) — and a
**stored type**: `Proposal` holds a delegate's declarations verbatim in the
snapshot, `#[serde(default, rename = "declare")] declarations: Vec<Declaration>`
(`src/design_run/delegation.rs:68-69`). That was deliberate and DRY — the code
says so: held as the wire type so acceptance runs the declarations through the
same engine rather than a second interpreter.

The repair is to stop storing `Declaration`, normalising to a separate stored
form at the delegation boundary, so the wire type can be tightened without
converting previously-readable stored state into a parse failure.

## Why it is deferred, and what expires

`DEC-243` (strictness refuses the never-known; a retired-member roster carries
the once-known) lists this as the clean layering fix that *dissolves* the
question rather than answering it, and defers it as named debt: no live snapshot
exercises the overlap. `EVD-027` measured it — all 16 live design-run snapshots
read `delegation = []` on 2026-09-14.

That is a measurement, not a guarantee, and it stops being true the first time
delegation is used in anger.

## Why the `SL-259` floor does not cover this

`RV-365` `F-3` (raised by an external adversarial reviewer against `SL-259`'s
design) established that the degradation floor `SL-259` introduces cannot be
stretched to cover the gap:

- `Declaration` carries `deny_unknown_fields`, so a stored proposal declaration
  written by an older binary — using a key a later binary retires — fails the
  **whole snapshot parse**. That is `ISS-315`'s failure mode, the one `SL-259`
  exists to end.
- The floor `SL-259` builds is `StoredRow`/`RawRow`, confined to
  `ChangeLog.rows`.
- Extending that floor to the delegation group looks cheap but is **forbidden by
  `DEC-249`** — degrade what is history, refuse what is state. An unapplied
  proposal awaiting acceptance is state.

So for stored declarations the retired-member roster is not the weakest guard,
it is the **only** guard — and `DEC-243`'s own rationale is that a roster is
discipline, not construction. `ISS-315` is the record of that discipline failing
three times, once unrostered.

## Trigger

Do this **before** delegation carries its first real proposal, not after. Until
then the exposure is genuinely zero and the debt is honest. The moment a live
snapshot holds a non-empty `delegation`, the roster becomes load-bearing for
snapshot legibility with nothing underneath it.

Re-run `EVD-027`'s probe to check: if any live snapshot reads a non-empty
`delegation`, this item is due.
