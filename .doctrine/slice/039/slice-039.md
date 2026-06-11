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

1. **Backlog item→item dependency edges.** Extend the backlog model with an
   authored, outbound-only (ADR-004) dependency relation between items (working
   name `depends_on`); reciprocity is derived, never stored. TOML in
   `[relationships]`; template + renderer + a CLI verb to set/show it.
2. **A uniform priority signal** sufficient to order by. Definition is an open
   question (OQ-1) — risk facets give `likelihood`×`impact`; other kinds need a
   rank or a defined default. Settle in `/design`.
3. **cordage adapter** (new doctrine-side module). The vocabulary layer:
   - `ItemId ↔ NodeId` bimap (both directions; callers never juggle raw tokens);
   - mint the named dependency overlay + priority channel **once**, retain the
     handles (no throwaway-builder hack);
   - **domain newtypes** so a semantically-wrong edge won't compile — the R-C
     kill (OQ-5);
   - a narrow surface: `add_dependency(a, b)`, an ordered-item list, `priority_of`.
4. **Ordered rendering.** `backlog list` (or a new verb) emits the
   dependency-topological, priority-tiebroken order, deterministically.
5. **Harvest the R-C interface rev.** Record what cordage API bend (if any) real
   use demanded — durable note + `/record-memory`; feed it back to cordage as the
   one budgeted non-breaking rev if warranted.

### Open questions (for `/design`)

- **OQ-1 — priority model.** Dep-topo + priority-tiebreak, or priority as a
  primary channel? How is priority defined per kind (risk likelihood×impact;
  default for the rest)?
- **OQ-2 — `explain()` coupling (the cross-slice coordination point).** Does
  ordering need cordage `explain()` ("why is X before Y?") or only
  `order_key`/`evaluate`? If it uses `explain()`, this slice's call-site **shapes
  RSK-002's explain-API change**; if it avoids it, the two stay decoupled.
  **Default posture: avoid `explain()` in v1** to keep SL-039 independent of the
  RSK-002 fix.
- **OQ-3 — cycle policy.** Author-time reject (deps must be acyclic) vs cordage's
  Evict/degrade tolerance. cordage supports both; backlog UX likely wants
  reject-at-author with a clear error.
- **OQ-4 — edge orientation.** `depends_on` mapping to cordage overlay direction
  (Along/Against) and what "ordered before" means (dependency-first vs
  dependent-first).
- **OQ-5 — newtype boundary.** The exact adapter surface that makes wrong-wiring
  unrepresentable — the concrete R-C kill.

### Affected surface (concrete)

- **New:** `Cargo.toml` (root) gains a path dep on `cordage`; a new adapter module
  in the doctrine crate (name TBD, e.g. `src/backlog_order.rs` or a
  `cordage_adapter` module).
- **Backlog model:** `[relationships]` schema (+ item-edge key), the backlog
  templates, entity renderer, and the backlog CLI command surface (`src/`).
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

Wire cordage's first consumer: give backlog items dependency edges, add a
doctrine-side vocabulary adapter (bimap + named handles + wrong-wiring-proof
newtypes), and render the backlog in dependency-topological, priority-tiebroken
order. Real-but-small use that earns the cordage API its keep and harvests the one
budgeted R-C interface rev. Independent of the cordage scale/perf streams.

## Follow-Ups

- The R-C interface rev (if real) → a small, non-breaking cordage follow-up slice.
- A backlog tech/product spec (SL-021) would later govern this capability
  retroactively.
- Closure intent: authored dep edges order correctly + deterministically;
  wrong-wiring won't compile (newtype proof); cordage unmodified (leaf invariant
  holds); the interface-rev finding recorded.
