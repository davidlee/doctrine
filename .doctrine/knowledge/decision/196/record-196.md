# The verdict kernel cuts at row identity

`SL-253` `inq-3` asked where the kernel/payload seam actually cuts. `DEC-190`
splits `crates/doctrine-control/src/conformance.rs` into a backend-neutral
verdict kernel and a per-mechanism payload, and the node named two references
pointing the wrong way — kernel to payload — as the obstacle.

## One of the two inlets was not real

`verify_over` (`:5270`) is already the injected seam. It takes the row set, the
row runner, and closures for auxiliary claims and unrowed observations; nothing
in it builds a `Fixture`. `verify` (`:5217`) is the assembly that does, and
`run_row` (`:5003`) already takes `&Fixture` explicitly. Placing `verify` and
`run_row` on the payload side dissolves that inlet with no restructuring at all.

## The seam

The kernel holds **identity and judgement**:

- the taxonomy — `Property`, `Axis`, `RowId`
- `RowVerdict` and the distinctions between its variants
- the verdict types
- `row_verdict` (`:3177`) — the two-arm algebra
- `verify_over`, keyed on `RowId` rather than on `Row`

The payload holds **construction** — `Row`, `Delta`, `ArmShape`, `Under`, `Arm`,
`run_arm`, `run_row`, `verify`, `PropertyRemoval`, `AuthorityGrant`, and the
whole fixture / probe / table surface. It closes over its own table to map an id
back to a row.

No kernel type names `Fixture`. Both backward references therefore vanish by
construction rather than by re-plumbing — which is the property being bought,
not a side effect of it.

## What happens to DEC-156

`DEC-156`'s one-property-removed control is an inference licence: the claim under
test is that the probe was refused *because of* the enforcement, and only a
capsule identical except for one loosened axis licenses it.

That licence splits cleanly along this seam. Its portable half is four lines of
algebra — probe read first; a failed probe is `Violated` whatever the control
did; a control that also held is `Unproven`; only probe-held-control-failed is
`Proven` — which is `SL-241`'s *a guard never seen to fire is not known to work*
made a type-level fact. That stays in the kernel.

Its other half is *how you build two capsules differing by one thing*, and that
is mechanism-keyed by `DEC-156`'s own third correction: a clause is written in
the vocabulary of what a capsule must not **have**, a row in the vocabulary of
the mechanism by which it **gets** it. The record's count moved 7 → 8 → 9 → 11 →
13 as channels were found, and every move was mechanism-specific. `DEC-189`
reaches the same conclusion from the other end.

So `SL-253` Objective 1's claim on the control discipline is honoured as
*algebra in the kernel, construction in the payload* — not as a literal claim on
`Arm` and `ArmShape`.

## Why `PropertyRemoval` and `AuthorityGrant` follow

By inspection rather than by argument. `PropertyRemoval`'s variants are
row-keyed, and one is documented as "expressed by enumeration rather than
subtraction, because bubblewrap has no `--share-pid`". `AuthorityGrant` has one
variant, row 14 — which `DEC-189` names among the four rows that lose their
host-facing delta under a hypervisor. Neither is portable vocabulary.
