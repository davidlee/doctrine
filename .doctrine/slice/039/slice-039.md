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

1. **Two backlog item→item edge types + the `triggers` rider field** — the PRD-009
   FR-010/FR-011 priority-engine enrichment, minted on the outbound seam. Extend
   the backlog model with two authored, outbound-only (ADR-004) item→item
   relations; reciprocity is derived, never stored. TOML in `[relationships]`;
   templates + renderer + CLI verbs to set/show them.
   - `needs` — a **hard** prerequisite, payload-free id list (authored on the
     dependent: the items that must land first). Cycles are an authoring error.
   - `after` — a **soft** manual-sequencing preference, inline-table list
     `{ to, rank }` (authored on the later item: the items it should come after).
     `rank` is the per-edge preference strength and the array order is the `age`
     source — both feed cordage's `EdgeAttrs(rank, age)` eviction key. Yields to
     `needs`; cycles tolerated.
   - `triggers` — an optional list of `{ globs, note }` architectural prefactor
     riders (PRD-009 FR-011/REQ-098). **Field only** this slice: parse + template +
     `show`. The actionability *mask* that consumes it is SPEC-001 D6, blocked on
     the open OQ-009 file-set source → deferred to **IMP-026**.
2. **A derived exposure signal** (explicitly *not* priority). No new authored
   field — risk *exposure* = `likelihood`×`impact` (each level 1..4 → product
   1..16) when assessed, else a baseline; every non-risk kind = the same baseline.
   It is the order's third tier, carried by the `NodeId` **fallback** (allocation
   order), not an overlay (OQ-1 settled — see the cross-level-drag rationale in
   `design.md` §10 A1). The related `list` filter/sort affordance is split out to
   IMP-021.
3. **cordage adapter** (new doctrine-side module). The vocabulary layer:
   - `ItemId ↔ NodeId` bimap (both directions; callers never juggle raw tokens);
   - mint the two named overlays (`needs`, `after`) **once**, retain the
     handles, compose them into one 2-layer `OrderSpec`; allocate nodes in
     `(exposure desc, created, id)` order so the fallback carries tiers 2–3 (no
     throwaway-builder);
   - **domain newtypes** that make wrong-wiring inexpressible *to callers* — the
     R-C kill (OQ-5): `NodeId`/`OverlayId` never escape the adapter, `ItemId` is
     the only token callers touch, so no raw cordage id can cross the boundary.
     (Internal handle-transposition is contained by named overlay fields, not by
     the type system — design E4/VT-10; the round-2 "won't compile" claim was
     retracted.)
   - a narrow surface: ingest the two authored edge maps + derived exposure,
     `ordered() -> Vec<ItemId>`, `overrides()` (the surfaced evictions), a
     dependency-cycle error.
4. **Ordered rendering.** A new `backlog order` verb emits the composed order
   (deps ≻ manual-seq ≻ exposure ≻ creation), deterministically, with an
   "overrides" block naming any soft edge cordage evicted. Leaves `backlog list`
   (and its goldens) untouched.
5. **Harvest the R-C interface rev.** Record what cordage API bend (if any) real
   use demanded — durable note + `/record-memory`; feed it back to cordage as the
   one budgeted non-breaking rev if warranted.

### Open questions — RESOLVED in `/design` (2026-06-11)

Full rationale in `design.md`; recorded here so scope and design stay aligned.

- **OQ-1 — ordering model.** *Resolved:* cordage-native composition, **no adapter
  sort**. Four tiers, hardest wins: **deps ≻ manual-seq ≻ exposure ≻ creation**.
  Tiers 0–1 are two overlay layers (`OrderSpec` edges → composed levels); tiers 2–3
  are the implicit `NodeId` fallback, with nodes allocated in `(exposure desc,
  created, canonical-id)` order. `ordered()` is longest-path over the merged graph
  (not lexicographic-by-layer — SL-036 F11); precedence = eviction authority.
  Exposure is the fallback, NOT an overlay (an overlay drags incomparable items
  across dep levels — `design.md` §10 A1). Uniform invariant: every edge `s→t`
  means "s before t", all layers `Along`. Both authored edges point at
  predecessors, so both flip to src-before-dst: `A.needs=[B]` and
  `A.after=[{to=B}]` each emit cordage edge `B→A`.
- **OQ-2 — `explain()` coupling.** *Resolved:* **avoid.** Ordering reads
  `order_key()` only; `explain()` is never called. SL-039 stays decoupled from
  RSK-002's explain-API change.
- **OQ-3 — cycle policy.** *Resolved:* split by edge type. `needs` =
  `CyclePolicy::Reject` + a hard error naming the cycle members. `after` =
  `CyclePolicy::Evict` + **info**: drop the min-key edge (now the genuine
  `(rank, age, src, dst)` key, not the A4 `(src,dst)` fallback), surface every
  dropped edge (`provenance().evictions()`) in the `order` render.
- **OQ-4 — edge orientation.** *Resolved:* uniform "src before dst", all layers
  `Along` (see OQ-1).
- **OQ-5 — newtype boundary.** *Resolved:* an `ItemId` newtype + one adapter
  struct owning `{Graph, ItemId↔NodeId bimap, 2 named OverlayIds}`; raw cordage
  tokens never surface. NodeIds captured from `builder.node()` (never constructed).
- **OQ-6 — authored vocabulary (reconcile, 2026-06-11).** *Resolved:* the edge +
  rider field names follow **PRD-009 FR-010/FR-011** (the minting product spec),
  not slice-local invention: hard `needs`, soft `after = [{ to, rank }]`, and the
  `triggers` rider. Supersedes the original `depends_on`/`before` (caught
  post-lock: the names diverged from PRD-009 with no rationale, and SL-036 design
  had anticipated `needs`/`after`/`triggers`). `after`'s `{ rank, age }` now drives
  cordage's `EdgeAttrs` eviction (the original A4 `(src,dst)` tie-break hack is
  retired). `triggers` is field-only here; its mask is IMP-026 (blocked on
  SPEC-001 OQ-009).

**Altitude (decided early):** the two item→item edges need **no new ADR** — both
are ADR-004 outbound-only relations (authored on one canonical side, reverse
derived). The cordage mapping and ordering model are slice-local design.

### Affected surface (concrete)

- **New:** `Cargo.toml` (root) gains a path dep on `cordage`; a new adapter +
  render module `src/backlog_order.rs`.
- **Backlog model:** `[relationships]` schema gains two item-edge keys
  (`needs`, `after`) + the `triggers` rider field, both backlog templates, the
  entity renderer (`show`), and the backlog CLI surface — a set verb per edge type
  plus the new `backlog order` verb (`src/backlog.rs`).
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
`needs` + soft `after = [{ to, rank }]`) plus the `triggers` rider field, add a
doctrine-side vocabulary adapter (bimap + named handles + wrong-wiring-proof
newtypes), and render the backlog in the
composed order **deps ≻ manual-seq ≻ exposure ≻ creation** via a 2-layer cordage
`OrderSpec` + its native fallback — the engine's headline composition, exercised
end-to-end. The
adapter does no sorting; cordage composes. Real-but-small use that earns the
cordage API its keep and harvests the one budgeted R-C interface rev. Independent
of the cordage scale/perf streams.

## Follow-Ups

- The R-C interface rev (if real) → a small, non-breaking cordage follow-up slice.
- A backlog tech/product spec (SL-021) would later govern this capability
  retroactively.
- Closure intent: authored dep edges order correctly + deterministically;
  no raw cordage id crosses the adapter boundary (the bounded R-C kill — callers
  cannot pass a `NodeId`/`OverlayId`; internal transposition contained by named
  fields, design E4/VT-10, *not* a "won't compile" proof); cordage unmodified
  (leaf invariant holds); the interface-rev finding recorded.
