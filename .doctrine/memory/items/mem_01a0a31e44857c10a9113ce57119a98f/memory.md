# Guard a declaration/construction pair at the seam that sees both

`design_run` is full of pairs where one table **declares** a shape and scattered
sites **construct** against it: `ChangeEvent::payload_terms` vs every
`PayloadTerm::{token,label,digest,prose}` call; `Declaration::WIRE_KEYS` vs what
`declare_node` actually reads; `payload_contract`'s inventory vs the wire structs.

**They drift, and the drift is silent** — both halves compile, and the wrong half
is admitted against a bound or a rule its own declaration does not claim for it.

Three instances, two slices apart:

| pair | drift | how it surfaced |
|---|---|---|
| `StepDischarged` declares `Step` as `Token`; the site built a `label` | a 17-byte step id parsed, blocked its edge, could never be discharged | `SL-233` `PHASE-08` `F-P08.2` — fixed **at the instance** |
| same event declares `Outcome` as `Token`; the site builds a `label` | the term was admitted against 16 B while the declaration claimed 32 B | `ISS-290`, still live **three slices later** — the `Step` fix never asked what else wore the shape |
| `Declaration`'s wire keys vs their honouring kind, then their honouring state | a key spellable everywhere, honoured at one kind/state, silently dropped elsewhere | `ISS-318`, then `ISS-327` — the same defect rotated onto a second axis |

## The move

A one-time grep closes the instance. **A check at the seam closes the class.**

1. Find the seam where *both* halves are in scope. For a payload term that is
   not the term's constructor — a `PayloadTerm` has no event until a row pairs
   them — but `ChangeEvent::shaped`, which every `Pending` constructor calls.
2. **Fuse the check into a method the path already has to call**, rather than
   adding one beside it. `shaped` absorbed `ordered` for exactly this: a
   separate `admits()` leaves a future constructor free to call only the sorter.
3. Make the refusal a typed variant, not a `debug_assert` — the drift is
   reachable in release, and `ISS-290` is what the silent version looks like.
4. Test it **quantified over the vocabulary** (every event × its own declared
   pairs × `ValueKind::ALL`), not over the one live instance. Assert both
   sides per cell: the declared pair is admitted, the others refused. A refusal
   that fires everywhere is a ban, and only the control catches it.

## The oracle problem, and its answer

A test asking "does the checker agree with the table?" proves only that the
table equals itself — it stays green under a table whose rows are **swapped**
([[mem.pattern.testing.mapping-oracle-lives-below-the-check]] states the general form).

Here the answer is free: once the seam refuses, **the engine's own real rows
are the differencing**. Reverting `Outcome` to `Token` left the unit matrix
green and turned five `e2e_design_runbook` tests red with
``step_discharged` declares `outcome` as token, not label``. So do not write a
test asserting the table's contents; prove it with a mutation probe against the
suite that exercises the engine.

Related: [[mem.pattern.review.sweep-defect-class-not-instance]].
