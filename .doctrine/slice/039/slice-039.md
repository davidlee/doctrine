# Backlog dependency ordering — item edges + cordage adapter

## Context

First **real consumer** of `cordage` (SL-036, governed by SPEC-001). cordage is a
pure-leaf, zero-dependency graph engine — a tree + typed directed (DAG) overlays,
deterministic build→query, ordering edges by **opaque** `rank`/`age` attrs. Its
design (D1) splits ownership: **core owns mechanism, consumer owns vocabulary.**
This slice is the *consumer half* — and cordage has had none until now: it is a
workspace member but **no crate depends on it yet**. SL-039 wires the first
`doctrine → cordage` path dependency (the ADR-001 layering edge: doctrine depends
on the leaf, never the reverse).

Why the backlog, why now:
- It is a **real, small dataset** — tens of items, shallow/sparse deps. That is
  far below every confirmed scale cliff (RSK-002 explain, RSK-003 overflow +
  eviction quadratic, RSK-004 evaluate quadratic), so SL-039 is **independent of
  SL-038** (scale harness) and the downstream fix slice. The unverified H1
  performance posture is irrelevant at this size.
- It is the **intended driver of the budgeted R-C interface rev**: cordage's
  design Lock reserved "the first real consumer drives one cheap, non-breaking
  interface rev, cheap while workspace-internal." Real use here surfaces the one
  API bend cordage needs, and the consumer-side **domain newtypes dissolve R-C**
  (opaque-handle friction + semantically-wrong-but-type-valid wiring) by making
  wrong wiring *unrepresentable*, not merely discouraged.

The data gap that defines this slice: backlog items today carry **no item→item
edges** — `[relationships]` is `slices` / `specs` / `drift` only, and the
`[[RSK-003]]`-style links in item prose are unstructured wikilinks. Priority is
**non-uniform**: risks carry `likelihood`×`impact`; issues / improvements /
chores / ideas carry no structured rank. So there are *no ordering inputs to
consume yet* — introducing them is half this slice's work.

## Scope & Objectives

1. **Two backlog item→item edge types.** Extend the backlog model with two
   authored, outbound-only (ADR-004) item→item relations; reciprocity is derived,
   never stored. TOML in `[relationships]`; templates + renderer + CLI verbs to
   set/show them.
   - `depends_on` — a **hard** prerequisite (authored on the dependent). Cycles
     are an authoring error.
   - `before` — a **soft** manual-sequencing preference (authored on the earlier
     item: the items it should precede). Yields to `depends_on`; cycles tolerated.
2. **A derived priority signal.** No new authored field — priority is *derived*:
   risk = `likelihood`×`impact` (each level 1..4 → product 1..16) when assessed,
   else a baseline; every non-risk kind = the same baseline. The order's third
   tier (OQ-1 settled).
3. **cordage adapter** (new doctrine-side module). The vocabulary layer:
   - `ItemId ↔ NodeId` bimap (both directions; callers never juggle raw tokens);
   - mint the three named overlays (`depends_on`, `before`, priority) **once**,
     retain the handles, compose them into one `OrderSpec` (no throwaway-builder);
   - **domain newtypes** so a semantically-wrong edge won't compile — the R-C
     kill (OQ-5): `NodeId`/`OverlayId` never escape the adapter, `ItemId` is the
     only token callers touch;
   - a narrow surface: ingest the two authored edge maps + derived priority,
     `ordered() -> Vec<ItemId>`, `overrides()` (the surfaced evictions), a
     dependency-cycle error.
4. **Ordered rendering.** A new `backlog order` verb emits the composed order
   (deps ≻ manual-seq ≻ priority ≻ creation), deterministically, with an
   "overrides" block naming any soft edge cordage evicted. Leaves `backlog list`
   (and its goldens) untouched.
5. **Harvest the R-C interface rev.** Record what cordage API bend (if any) real
   use demanded — durable note + `/record-memory`; feed it back to cordage as the
   one budgeted non-breaking rev if warranted.

### Open questions — RESOLVED in `/design` (2026-06-11)

Full rationale in `design.md`; recorded here so scope and design stay aligned.

- **OQ-1 — ordering model.** *Resolved:* cordage-native multi-layer `OrderSpec`,
  **no adapter sort**. Four tiers, hardest wins: **deps ≻ manual-seq ≻ priority ≻
  creation** (creation = the implicit `NodeId` fallback, with nodes allocated in
  `(created, canonical-id)` order). Priority is derived-only (objective 2). The
  uniform overlay invariant: every edge `s→t` means "s before t", all layers
  `Along`; the one flip is authored `A.depends_on=[B]` → cordage edge `B→A`.
- **OQ-2 — `explain()` coupling.** *Resolved:* **avoid.** Ordering reads
  `order_key()` only; `explain()` is never called. SL-039 stays decoupled from
  RSK-002's explain-API change.
- **OQ-3 — cycle policy.** *Resolved:* split by edge type. `depends_on` =
  `CyclePolicy::Reject` + a hard error naming the cycle members. `before` =
  `CyclePolicy::Evict` + **info**: drop the min-key edge, surface every dropped
  edge (`provenance().evictions()`) in the `order` render.
- **OQ-4 — edge orientation.** *Resolved:* uniform "src before dst", all layers
  `Along` (see OQ-1).
- **OQ-5 — newtype boundary.** *Resolved:* an `ItemId` newtype + one adapter
  struct owning `{Graph, ItemId↔NodeId bimap, 3 OverlayIds}`; raw cordage tokens
  never surface. NodeIds captured from `builder.node()` (never constructed).

**Altitude (decided early):** the two item→item edges need **no new ADR** — both
are ADR-004 outbound-only relations (authored on one canonical side, reverse
derived). The cordage mapping and ordering model are slice-local design.

### Affected surface (concrete)

- **New:** `Cargo.toml` (root) gains a path dep on `cordage`; a new adapter +
  render module `src/backlog_order.rs`.
- **Backlog model:** `[relationships]` schema gains two item-edge keys
  (`depends_on`, `before`), both backlog templates, the entity renderer (`show`),
  and the backlog CLI surface — a set verb per edge type plus the new
  `backlog order` verb (`src/backlog.rs`).
- **No change** to `crates/cordage/**` — consumer-only; cordage stays a pure leaf
  (verified by `cargo tree -p cordage` showing it alone).

## Non-Goals

- **No cordage perf work.** The cliffs (RSK-002/003/004) are large-scale; SL-038
  (reds) and the downstream fix slice own them. Irrelevant at backlog size.
- **No cordage `explain()`-API change.** At most a coordination note for RSK-002
  (per OQ-2). This slice does not redesign cordage's surface.
- **No backlog product/tech spec.** Authoring a spec for the backlog capability is
  SL-021 territory, not this change.
- **No general graph features** beyond what ordering needs — ride cordage's
  existing public API.

## Summary

Wire cordage's first consumer: give backlog items two item→item edge types (hard
`depends_on` + soft `before`), add a doctrine-side vocabulary adapter (bimap +
named handles + wrong-wiring-proof newtypes), and render the backlog in the
composed order **deps ≻ manual-seq ≻ priority ≻ creation** via a multi-layer
cordage `OrderSpec` — the engine's headline composition, exercised end-to-end. The
adapter does no sorting; cordage composes. Real-but-small use that earns the
cordage API its keep and harvests the one budgeted R-C interface rev. Independent
of the cordage scale/perf streams.

## Follow-Ups

- The R-C interface rev (if real) → a small, non-breaking cordage follow-up slice.
- A backlog tech/product spec (SL-021) would later govern this capability
  retroactively.
- Closure intent: authored dep edges order correctly + deterministically;
  wrong-wiring won't compile (newtype proof); cordage unmodified (leaf invariant
  holds); the interface-rev finding recorded.
