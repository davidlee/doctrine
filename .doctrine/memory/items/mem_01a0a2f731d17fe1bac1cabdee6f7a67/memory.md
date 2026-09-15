# An earlier phase can close a later phase's criterion

A plan decomposes a design into phases, and the phases are written **before any
of them runs**. Where two phases repair overlapping defects, an earlier phase's
fix can dissolve part of a later phase's scope — and nothing in the plan
structure records that it might.

## The instance

`SL-259` decomposed one design into six phases by *leg*. `PHASE-04` (leg 3's
state axis, `DEC-246`) named four state-inert wire keys to refuse. `PHASE-01`
(leg 2, `DEC-248`) collapsed `declare_node` onto a single row-producing path over
two priors, to stop a `needs` edge declared at node creation landing silently.
The same collapse made a `lifecycle` declared at creation land too — which was
the whole of `PHASE-04`'s `lifecycle` cell. Leg 2 repaired a leg 3 defect as a
side effect, three phases before leg 3 ran.

`PHASE-04`'s `EN-2` said *"the four cells are still the four cells"* and listed
the anchors to re-probe: the four key constants and the kind-axis table. All of
them were live. The count was the only false claim, and it was the one thing the
criterion did not tell you how to check.

## Why the usual guards miss it

- The **entrance criterion** names anchors, and anchors survive — the file:line
  citations were fine. A criterion that says "re-verify X" cannot make you
  re-verify Y.
- **Tests** do not catch it: `PHASE-01` left every suite green, because making a
  key honoured is not a regression of anything.
- The **design** still states the four, so reading the design agrees with the
  plan. Both are pre-implementation documents; they agree because neither has run.
- Nothing in `plan.toml` can express *"PHASE-01 may close PHASE-04's cell 2"* —
  there is no cross-phase interaction field, and no reader is looking for one.

## How to apply

At `/phase-plan`, when the phase's criteria assert a **count** or an
**enumeration** — N cells, N call sites, N rows — re-derive it from the code
rather than re-probing the anchors the criterion names. The cheap form is a table
over the full domain (all fifteen wire keys × both states, not just the four the
plan lists), which costs a few greps and either confirms the count or hands you
the amendment.

Run that check first where an earlier phase of the **same slice** touched the
same subsystem. That is the condition that makes the count mobile; two unrelated
slices rarely do this to each other.

When the count has moved, amend the criteria — a field edit, ids untouched — and
record the cause where the next reader will meet it (`plan.md`'s
verification-posture section is where `SL-259` put it). Do **not** restore the
missing cell to preserve the count: re-refusing `lifecycle` would have regressed
`PHASE-01`, which is
[[mem_019fbcdbd2dc70d1b4aec3209955ffe1]] — a repair travelling past its correct
point and landing in a defect facing the other way.

The locked design still states the old count. That is authored-truth drift for
`/audit` and `/reconcile`, not something to fix mid-phase.

Related: [[mem.pattern.doctrine.vt-test-file-is-a-guess-retarget-it]] — the same
discipline on a plan's `test_file` mandates, and
[[mem_01a00d11d24d70a1bf531fe6561c426b]] — a design's call-site census ages.
