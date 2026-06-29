# SL-176 — RESOLVED DECISION: priority `fulfils` effect (value burndown)

**Status: RESOLVED 2026-06-29 (design session 2).** `design.md` is authoritative again — it
**supersedes this file**. This is the investigation trail; the binding outcome is in the
design ledger (D-priority-burndown / D-burndown-denomination / D-burndown-lifecycle /
D-value-floor-sibling), the rewritten §A′.1 row 1 + "Priority burndown" spec, and R10/R12.

> **FINAL RESOLUTION (supersedes the "Resolution direction" + TL;DR below — those captured an
> earlier *additive-credit + degree-weight* model that was itself replaced).**
>
> The `fulfils` priority effect is a **value BURNDOWN**, not additive optionality, and not the
> degree-weighted credit drafted below:
> 1. **Subtractive, value-denominated.** A backlog item's value is *reduced* by the raw `value`
>    of the slices fulfilling it (item-4, slice-3 → residual 1). `delivered = Σ gate(status)·
>    value(slice)`; item `value_dim` offset proportionally, clamped ≥ 0.
> 2. **Off the additive pass.** `Slices` removed from both `REF_LABELS`+`CONSEQUENCE_LABELS`;
>    `Fulfils` joins `REF_LABELS` ONLY (overlay for `in_edges`); a NEW post-pass does the
>    subtraction. `CONSEQUENCE_LABELS` only *adds* — wrong sign/direction for burndown.
> 3. **Degree is OUT of scoring.** Locked binary `Degree{Full,Partial}` can't carry a fraction;
>    ADR-016 §2 (derivable-not-relational) bars a coverage fraction on the edge. The named
>    partial-weight constant drafted below is **dropped**. Degree keeps display + IMP-210 only.
> 4. **Lifecycle-gated.** `planned`/draft slices burn 0; `in_progress`/`completed` burn full.
> 5. **Non-conserving** across multi-item (per-item `in_edges` sum) — deliberate, leverage-like.
> 6. **Default value 1.0** for value-bearing actionable kinds {slice, backlog} (records
>    EXCLUDED) → **sibling slice**; SL-176 broadened to own burndown (Option 2, user-locked);
>    soft dependency only. Coverage-% derived display → deferred follow-up.
>
> The old `slices`→optionality credit is **dropped, not replaced** (User-vetoed OK).

---

## TL;DR

Design **R10** + **§A′.1 row 1** claim: re-point `priority/graph.rs` `Slices`→`Fulfils` in
the label sets and "optionality numbers stay identical — a `fulfils` inbound credits
optionality exactly as a `slices` reference did." **That claim is false** (proven below).
The numbers move because the edge direction flips. But — per user guidance — **preservation
was never the goal**; the correct behaviour is a *deliberate change*. The resolution
direction is settled; one scope question remains.

---

## The mechanism (verified in source this session)

Priority score per entity = `base + leverage + optionality` (`src/priority/graph.rs:171`).

- **`base_score`** (`graph.rs:102-134`): `value_dim + risk_dim`, computed from the entity's
  **OWN** facets only. `value_dim` (`:114-122`) = `coeff.value · value · kind_weight · tag /
  est_cost`; it is **0 when the entity has no `value` facet**. So `base ≈ 0` for an entity
  carrying no value/risk facet.
- **`optionality`** (the consequence post-pass, `graph.rs:595-628`): for each node N, sum over
  N's **in-edges** under the `CONSEQUENCE_LABELS` overlays, each contributing `ref_coeff ·
  base(source)`. **Optionality credits the edge TARGET, proportional to the SOURCE's base.**
  Witness test `nodes_authoring_no_dep_seq_carry_no_edges` (`graph.rs:1038-1078`): `ISS-001`
  (base ≈ 3.846) points at `ISS-002`; asserts `optionality[ISS-002] == base(ISS-001)`.
- **Label sets** (`graph.rs:183-205`): `Slices` is in BOTH `REF_LABELS` (overlay build,
  `:318` — mechanical, harmless to swap) and `CONSEQUENCE_LABELS` (the scoring — the problem).

## Why R10 is false — the direction flip

Optionality flows source→target (credits target ∝ source base). The migration reverses the
edge:

- **Today `slices` (item→SL):** source = backlog item, target = slice. The **slice** is
  credited ∝ `base(item)`. Value facets live on items, so this is a **live** signal: a slice
  is surfaced by the value of the items pointing at it.
- **Post-migration `fulfils` (SL→item):** source = slice, target = item. The **item** is
  credited ∝ `base(slice)`. Different node, different multiplier.

A `slices=["SL-x"]` fixture re-authored as a `fulfils` edge does **not** reproduce the old
number on the old node — it can't, because `CONSEQUENCE_LABELS` optionality is hardwired to
`in_edges` (target-credit) and the target is now a different entity. So "green on equivalent
fixtures, numbers identical" (R10's proposed proof) is unachievable.

## Empirical snapshot (do NOT design around this — see user guidance)

- **0 of 349** slice files carry a `value`/`risk` facet (grep, 2026-06-29).
- ~49 of 330 backlog files carry value/risk; items routinely carry `[value]`.

My initial (wrong) inference was "`base(slice) ≈ 0` ⇒ crediting the item is a dead signal."
**User corrected this** (see below): the sparsity is low *uptake* of a recently-working
facet, not a design truth. Slices can and should optionally carry est/value (rare vs backlog,
but valid). The mechanism must be correct for when they do.

---

## User guidance (decisive — 2026-06-29)

Verbatim intent:

1. "slices should be (and are) optionally able to take est / value — though it can be
   expected to be rare relative to backlog." → `base(slice)` is legitimately nonzero; do not
   optimise for today's sparse corpus.
2. "inbound edge counting is a bullshit heuristic; uptake of est / value is low because it's
   only recently working, not because it's not correct." → Do not fetishise the current
   optionality mechanism or exact-number preservation. Correct semantics > preserved numbers.
3. "adding est / value for a new slice should have sane effects on the priority of a backlog
   item it is partially or wholly delivering." → **This is the directional requirement.**

(3) sets the direction: value flows **slice → item** along `fulfils`. The **item** is
credited from the **slice's** facets, **scaled by degree** (partial vs whole). That is the
in-edges orientation — i.e. the design's naive label-swap actually produces the *correct*
direction. My `/consult` option (d) (credit the slice / read fulfils reversed) was BACKWARDS
and is rejected.

---

## Resolution direction (settled by the above)

1. **Keep the in-edges orientation** for `fulfils` in `CONSEQUENCE_LABELS`: the backlog
   **item** is credited from the fulfilling **slice's** base. (= what `Slices`→`Fulfils`
   swap mechanically does.)
2. **Delete R10's "numbers identical / behaviour-preserving" claim.** Reframe §A′.1 row 1 +
   R10 as a **deliberate correctness change**: an item's priority now reflects the facets of
   the slices fulfilling it. No preservation proof; the proof obligation becomes a
   *behaviour-change* assertion.
3. **Weight the contribution by degree** (the user's "partially or wholly"): optionality
   contribution becomes `ref_coeff · weight(degree) · base(slice)`, with `full` at full
   weight and `partial` at a reduced **named-constant** factor (STD-001 — no magic number).
   - This makes the **degree facet a real priority consumer**, not just display/`inspect`.
   - Does NOT violate "degree does not aggregate" (D-degree-default / ADR-016 §2): that bans
     merging two partials into a full; this is per-edge weighting, not aggregation.
4. **Accept the loss** of the old "slice credited by the value of items it served" signal.
   User did not ask to keep it and called inbound-counting crude. **Flagged for final veto**
   — confirm with user, but proceed assuming acceptable.

### New verification (replaces R10's preservation proof)

Priority fixtures asserting the *changed* behaviour:
- A slice gains `value`/`est` → the backlog item it `fulfils` sees its optionality/priority
  move sanely (not zero, correct sign).
- A `partial` fulfils contributes strictly less than a `full` one (degree weighting witnessed).
- `originates_from` (provenance) does **NOT** feed optionality (the conflation fix — the real
  reason the old behaviour was wrong: `slices` mixed provenance into the priority signal).

---

## THE OPEN SCOPE QUESTION (for the fresh agent + user)

Degree-weighted optionality is a small but real **new scoring behaviour**: a named `partial`
weight constant + threading the `fulfils` degree into the consequence pass (`graph.rs:595-628`).
Two ways to take it:

- **(A) In scope for SL-176** — re-point `fulfils` *with* degree weighting now. Natural home
  (degree is this slice's new facet; it is exactly the "partial vs whole" effect the user
  described). **Author's recommendation.**
- **(B) Defer the weighting** — re-point unweighted now (item credited from slice base, degree
  ignored in scoring), spin "degree-into-priority" as a follow-up backlog item (sibling to
  IMP-210, the close-cascade degree consumer).

Resolve (A) vs (B) with the user, then proceed.

---

## What the fresh agent must do

1. Confirm (A) vs (B) and the veto on dropping the slice-credited-from-item signal.
2. Revise `design.md`:
   - **R10** (§D Risks) — delete the false preservation claim; restate as deliberate
     correctness change + (if A) degree weighting.
   - **§A′.1 row 1** (priority consumer) — rewrite "preserve the scoring numbers" → the new
     item-credited-from-slice-facets semantics; note provenance excluded.
   - If (A): note the degree-weight named constant + the consequence-pass thread in §A.6 /
     §A′.1, and add it to the §D phasing (PHASE-03).
3. Revise `plan.toml` PHASE-03:
   - **EX-4** currently says "optionality scoring numbers identical (R10)" — WRONG; rewrite to
     the behaviour-change assertion (+ degree weighting if A).
   - **VT-3** mandate (`test_file = src/priority/graph.rs`, keywords `["Fulfils","optionality"]`)
     — re-aim at the new fixtures (slice-facet→item-priority; partial<full).
4. Resume `/plan`: re-read the critical pass, then materialise phases (`doctrine slice phases
   176`) → `/phase-plan` PHASE-01 → `/execute`.

## Anchors (verified this session, ~2026-06-29 — re-verify against fresh build)

- `src/priority/graph.rs:171` score = base+leverage+optionality; `:102-134` base_score (own
  facets); `:60-71` base = value_dim+risk_dim; `:183-196` REF_LABELS (Slices); `:199-205`
  CONSEQUENCE_LABELS (Slices); `:318` REF overlay build; `:595-628` optionality post-pass
  (in_edges, credit target ∝ base(source)); `:529` leverage (out_edges, dep overlay); witness
  test `:1038-1078`.
- Other live `Slices` consumers (design §A′.1, all in scope): `backlog.rs:1420`/`1574`/`2201`/
  `~961`, `lazyspec.rs:771` (`map_edge`). `fulfils` is SL→BACKLOG; `slices` was BACKLOG→SL.
- Design touch-points: `design.md` R10 (§D Risks), §A′.1 row 1, §A.6; `plan.toml` PHASE-03
  EX-4 / VT-3.

## Note (NOT a backlog item unless user asks)

The user's "inbound edge counting is a bullshit heuristic" is an editorial aside about the
crudeness of the optionality mechanism overall. Earlier I proposed a backlog item for
"base(slice)≈0 makes several CONSEQUENCE_LABELS near-dead" — the user reframed that as low
uptake, not deadness, so **do not file it** without asking. If anything is captured, it's
"revisit the optionality heuristic," which is RFC-003-deferred territory, not SL-176.
