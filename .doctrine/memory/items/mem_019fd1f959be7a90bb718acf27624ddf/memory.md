# Reconcile-stage REV delivery never reads conformant

A slice that declares a governance path (`.doctrine/spec/**`, `adr/**`, …) as a
`design-target` selector and discharges it through a `REV` at `/reconcile` will
see `slice conformance` report that selector **undelivered** forever. It is
structural, not an undischarged obligation.

## Why

`slice record-delta` binds a source-delta row to a `PHASE-NN` — that is the only
attribution the registry has, and there is no reconcile-stage row. `ADR-013` and
`/reconcile` route governance/spec truth through a `REV`, which lands *after* the
last phase is `completed`. So the discharging commit is outside every recorded
boundary, and the algebra cannot see it.

Both obvious repairs are wrong:

- **Widening the last phase's boundary** to swallow the reconcile commit
  falsifies the phase record.
- **`slice selector rm`** is the sanctioned fix for a *spurious* undelivered row.
  This one is not spurious — the target was genuinely declared and genuinely
  delivered, and dropping the selector erases the promise audit checks against.

## How to apply

At `/close`, an `undelivered` row pointing at a governance path the slice's `REV`
already landed is **explained, not a dropped deliverable**. Confirm the REV is
`done` and its target validates (`doctrine spec validate SPEC-NNN`), record the
residual in the RV's `## Reconciliation Outcome`, and close. Do not widen a
boundary and do not remove the selector.

First hit at `SL-244` / `RV-345` `F-2`: `REV-048` landed the owed `SPEC-029`
amendment in both tiers, and `slice conformance 244` still reported
`undelivered (1): .doctrine/spec/tech/029/**`. Filed as `IMP-292` **Defect 4**,
beside the three sibling conformance-signal defects.

Distinct from [[mem_019f239c569b75239987428d47b11f8f]], which is the *undeclared*
cell for a REV-only slice whose deliverable matches no selector at all. Here the
selector exists and matches; what is missing is a stage the row can bind to.
