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
   inbound** (ADR-004), not a stored label. The degree is a **facet** (a quantity), not
   a role and not a coverage substrate.
3. **Migration (SL-149 redux).** Retire the `slices` label; relabel existing edges.
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
  as a closure hint is a consumer change. Decide in design whether a minimal hint is
  in-scope or spun out; the cascade must stay **hint-not-auto** (F-6) regardless.
- **Axis D** (`part_of` / altitude), **`influences` planes**, **`exclusive_with`**,
  **prose-hunt**, and the **vocabulary-evolution feedback loop** — all deferred (RFC-003
  Deferred set).
- **Sub-roles** on `originates_from` (`scoped` vs `follow_up`) — one neutral role now;
  sub-roles only if an edge later demands the distinction.

## Affected surface (coarse — `/design` pins the exact touch-set)

- `src/relation.rs` — `RelationLabel` + `Role` enums, `RELATION_RULES` table, `lookup`,
  `Tier`/`LinkPolicy`/`TargetSpec`, the tier-1 migration predicate.
- `src/relation_graph.rs` — `outbound_for`, inbound/reciprocity derivation (the derived
  `fulfils` inbound replacing stored `slices`), overlay allocation.
- `src/commands/relation.rs` — `link`/`unlink`/`inspect` surfaces; the `--degree` axis.
- The **migration** path (SL-149's edge-rewrite machinery — locate via SL-149).
- `install/templates/{slice,backlog}.toml` — scaffold rows referencing `slices`/`drift`.
- The 19 IMP-207 backlog `*.toml` (retcon targets) + any live `slices`/`drift` rows.
- Tests: relation rule-table goldens, migration e2e, lockstep enum-order tests.

## Risks / assumptions / open questions

- **`fulfils` storage + write seam.** Tier-1 with a degree column, or tier-2 typed
  payload (like `interactions`' free-text `type`)? `LinkPolicy` — `Writable` with a
  `--degree` flag, or a typed verb? Threading a *facet* (not just a role) through the
  seam is the novel work; SL-149 only added the role column. **Design decision.**
- **`scoped_from` → `originates_from`: rename vs add+migrate.** Rename the enum variant
  in place (one wire-name change + edge migration) vs add new role and migrate. SL-149
  precedent informs this.
- **Author-at-the-mutable-end** is a *new modelled property* — the authoring end is
  dictated by lifecycle state, not a fixed source kind. Is it enforced or convention?
  ADR-004 (outbound-only) still holds; this refines *which* end is outbound.
- **`{full, partial}` does not aggregate** (two partials ≠ full) — by design. Item
  completion over a set of inbound `fulfils` edges is a judgement, not arithmetic.
- **Behaviour-preservation gate** — the entity-engine suites must stay green; this rides
  the existing relation seam (no parallel implementation).
- **Dogfooding wrinkle.** This slice will itself carry `slices`/`originates_from` edges
  to IMP-207/IMP-149; its own migration must remain self-consistent.

## Verification / closure intent

- `references(originates_from)` legal and `scoped_from` edges migrated; `link`/`inspect`
  round-trip it; `relation census` shows the consolidated role.
- `fulfils` legal slice→backlog with a readable `{full, partial}` degree; the old
  `slices` reading renders as derived inbound on the backlog item.
- `slices` label retired; the 19 IMP-207 rows + `drift` entity rows migrated; no dangling
  or illegal edges (`doctrine validate` clean); `drift` non-entity rows untouched.
- Entity-engine suites green unchanged; rule-table goldens + migration e2e updated.
- Reconciliation authors the ratifying ADR; IMP-207 / IMP-149 closed.
