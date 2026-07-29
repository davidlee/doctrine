# IMP-356: Plan criteria: payload-vs-red-kind and data-shape fixture radius

Two authoring checks for `/plan` and `/phase-plan`, both found in SL-233
PHASE-06 (sheet findings F-2 and F-4). Siblings of the verification-mode
reachability check `/phase-plan` already gained — same class: a criterion that
is correct in substance but unreachable, or under-scoped, as written.

## 1. A criterion prescribing BOTH a payload shape and a red kind

PHASE-06 `VA-5` mandated the removed wire keys be sent *alongside a valid
`dispose`*; `VA-NC3` mandated the resulting red be a **wrong acceptance**. For
the `record` leg both cannot hold: that payload already hit a *second*
pre-existing refusal (`CheckpointDispositionConflict`), so its red is a wrong
*refusal reason*, not an acceptance. The `adopt_record` leg does red as an
acceptance and carried the criterion's intent; the worker found this and
reported it rather than fudging the assertion.

`VA-5`'s own rationale was sound — "without the `dispose` it would refuse as
`CheckpointDispositionMissing` and pass for the wrong reason". It simply did not
anticipate a second pre-existing refusal on the other leg.

Prescribing an input **and** an expected failure mode is two claims about the
incumbent, and the second is the one nobody executes before the phase runs.

**Check:** when a criterion fixes both a payload shape and a red kind, execute
that payload against the current tree at authoring time and confirm the observed
refusal is the one the criterion assumes — or name the alternative arm
explicitly.

## 2. A data-shape rule has a wider fixture radius than a field rule

PHASE-06 `EX-12` enumerated the fixtures carrying the removed annotation
spelling — and undercounted them (three named, four real; the fourth was
`tests/e2e_design_projection.rs:284`). Worse, `EX-13(b)` forced every section
body to **open with an ATX heading**, and no criterion anywhere listed a single
section-body fixture site. The real migration was roughly twice the enumerated
one, and one unlisted site hid a live trap:
`stored_change_row_keeps_full_reason_and_fingerprint` asserted
`sha256(b"draft two")` and silently depended on the old body bytes.

Enumerating the sites of a removed **field** tells you nothing about the sites
affected by a changed **data shape**.

**Check:** when a phase changes the shape of a value rather than the presence of
a field, the fixture radius is every construction site of that value, not the
grep for a field name. Say so in the criterion, or the worker inherits a list
that reads authoritative and is half the job.

## Provenance

- SL-233 PHASE-06 sheet `## Findings` F-2, F-4 (runtime, disposable).
- SL-233 `notes.md` `## Harvest` § Learned — the durable statement of both.
- Landed delta `564e8775`; phase concluded `d748bb1c`.

Related: the verification-mode reachability check (four instances across
PHASE-04 / PHASE-05, routed to `/phase-plan` rather than a fifth appended
criterion) — same defect class, same home.
