# Finish Axis B — slices/drift retirement

## Context

SL-149 / ADR-016 collapsed the work→**canon** noun-labels (`specs`, `requirements`)
into the universal `references` + closed-`Role` grammar. It left the work→**backlog**
noun-labels standing: **`slices`** and **`drift`**. They are the unfinished half of
Axis B — labels named after their target kind, not the verb. `slices` (BACKLOG→SL,
"addressed by") conflates three dimensions in one binary edge: **provenance** (who was
born from whom), **fulfillment** (this slice does the item's work), and **completion**
(how much). `drift` (BACKLOG→free-text) is the abused escape hatch for everything with
no structured outlet.

The decided direction is **RFC-003 § "Finish Axis B"** — read it; it is the
decision-of-record and this slice's brief. Governance ratification is deliberately
**deferred to this slice's reconciliation** (ADR amend/new): design settles and is
adversarially reviewed before any governance is touched. RFC-003 asserts no canon.

## Scope & Objectives

1. **Provenance → one neutral `references(originates_from)` role.** Generalize the
   shipped `references(scoped_from)` (SL→backlog) into `originates_from` meaning "I was
   born from the target," covering any work entity → its origin. Subsumes `scoped_from`
   and absorbs IMP-207's proposed `spawned_from` (backlog→SL). Authored at the
   **mutable / born end** (the live entity).
2. **Fulfillment + completion → new `fulfils` label + `{full, partial}` degree facet.**
   Slice → backlog. The old `slices` "addressed by" reading becomes `fulfils`' **derived
   inbound** (ADR-004), not a stored label. Degree is a non-keyed **facet** for inbound
   display + the IMP-210 close-cascade hint — **out of priority scoring** (D-burndown-denomination).
3. **`fulfils` priority effect → value BURNDOWN** (scope-broadened 2026-06-29, Option 2).
   A backlog item's priority reflects its **undelivered** value: a new `priority/graph.rs`
   post-pass *reduces* an item's value by the lifecycle-gated raw `value` of the slices
   fulfilling it (subtractive, value-denominated, degree-ignored, non-conserving).
   Replaces the old `slices`→optionality credit, which is dropped (design §A′.1 burndown
   spec, R10/R12). The default-1.0 value floor it relies on for valueless entities is a
   **separate sibling slice** (soft dependency).
4. **Migration (SL-149 redux).** Retire the `slices` label; relabel existing edges.
   Fold **IMP-207's 19-row retcon** (mislabelled `slices` → `originates_from`). Fold
   `drift` **entity** rows: "carved out from" → `originates_from`; "feeds into" → the
   dep/seq layer (`needs`/`after`). Retire `drift` for entity targets.

Discharges **IMP-207** (retcon rides this slice), **IMP-149** (`slices` ambiguity
dissolved), and **Axis C's completion hole** (`partially_addresses`, now the `fulfils`
degree facet).

## Non-Goals

- **Governance ratification** — the ratifying ADR (amend ADR-016/ADR-010, or a sibling)
  is authored at **reconciliation**, after design + adversarial review. Not in design/plan.
- **The non-entity-target edge** (memory / file / glob) — `drift`'s non-entity rows stay
  on `drift` and are deferred to that future edge (IMP-012 / IDE-015). `drift` is **not**
  fully retired by this slice.
- **The close-cascade consumer** — making `doctor` / `/close` *act* on `fulfils(full)`
  as a closure hint. **DECIDED (design): spun out → IMP-210.** Not in this slice; hint-not-
  auto (F-6) regardless.
- **Axis D** (`part_of` / altitude), **`influences` planes**, **`exclusive_with`**,
  **prose-hunt**, and the **vocabulary-evolution feedback loop** — all deferred (RFC-003
  Deferred set).
- **Sub-roles** on `originates_from` (`scoped` vs `follow_up`) — one neutral role now;
  sub-roles only if an edge later demands the distinction.
- **Default value 1.0 floor** for value-bearing actionable kinds {slice, backlog} (records
  excluded) — a **separate sibling slice**, not SL-176. Burndown (objective 3) works on
  explicitly-valued entities without it; the floor only governs the valueless case (soft dep).
- **`fulfils` coverage-% derived display** (`slice_value/item_value`) — deferred follow-up;
  scoring does not need it.

## Affected surface (coarse — `/design` pins the exact touch-set)

- `src/relation.rs` — `RelationLabel` + `Role` enums, `RELATION_RULES` table, `lookup`,
  `Tier`/`LinkPolicy`/`TargetSpec`, the tier-1 migration predicate.
- `src/relation_graph.rs` — `outbound_for`, inbound/reciprocity derivation (the derived
  `fulfils` inbound replacing stored `slices`), overlay allocation.
- `src/commands/relation.rs` — `link`/`unlink`/`inspect` surfaces; the `--degree` axis.
- `src/priority/graph.rs` — label-set membership (`Slices` out, `Fulfils` into `REF_LABELS`)
  + the new value-burndown post-pass (objective 3).
- The **migration** path (SL-149's edge-rewrite machinery — locate via SL-149).
- `install/templates/{slice,backlog}.toml` — scaffold rows referencing `slices`/`drift`.
- The 19 IMP-207 backlog `*.toml` (retcon targets) + any live `slices`/`drift` rows.
- Tests: relation rule-table goldens, migration e2e, lockstep enum-order tests.

## Risks / assumptions / open questions

The six open design questions are **resolved in `design.md`** (decision ledger). Summary:

- **`fulfils` storage** → Tier-1 + `degree: Option<Degree>` column + `link --degree`
  (mirrors SL-149's `role`; degree is non-keyed payload). Tier-2 typed rejected.
- **`scoped_from` → `originates_from`** → rename role variant in place; widen source/target.
  Parallel naming with the REV→RFC `originates_from` *label* accepted (different namespaces).
- **Author-at-the-mutable-end** → **convention** + source-set partial fence (`fulfils`
  SL-only structurally forces slice-end); no lifecycle enforcement (residual R9).
- **`{full, partial}` does not aggregate** — held; `None` degree ≡ full.
- **Close-cascade** → spun out (IMP-210). **Drift** entity rows → `originates_from` /
  `needs`-`after`; 5 free-text deferred.

Standing assumptions / risks:

- **Behaviour-preservation gate** — entity-engine suites stay green; `append_edge` upsert
  is a no-op for roleless/degreeless edges (SR-6). No parallel implementation.
- **Migration is per-edge, re-censused live** (SL-149 AR-1) — the prov-vs-fulfil split over
  82 `slices` is human judgement; IMP-207's 19 are reference, not input.
- **Dogfooding** — SL-176's own edges migrate self-consistently; the design-authored
  `IMP-210 references(concerns) SL-176` edge is provenance-shaped (may retcon).

**Follow-ups captured:** IMP-210 (close-cascade hint consumer), IMP-156 (create-time
`--originates-from` flag). Reconciliation: ratifying ADR + SPEC-018 + `relation-
vocabulary.md` + close RFC-003.

## Verification / closure intent

- `references(originates_from)` legal and `scoped_from` edges migrated; `link`/`inspect`
  round-trip it; `relation census` shows the consolidated role.
- `fulfils` legal slice→backlog with a readable `{full, partial}` degree; the old
  `slices` reading renders as derived inbound on the backlog item.
- `slices` label retired; the 19 IMP-207 rows + `drift` entity rows migrated; no dangling
  or illegal edges (`doctrine validate` clean); `drift` non-entity rows untouched.
- Entity-engine suites green unchanged; rule-table goldens + migration e2e updated.
- Reconciliation authors the ratifying ADR; IMP-207 / IMP-149 closed.
