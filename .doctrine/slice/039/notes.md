# SL-039 — implementation notes

Durable findings harvested from the disposable phase sheets. The runtime sheets
under `.doctrine/state/` are `rm -rf`-able; what must survive lives here.

## PHASE-01 (corrective) — PRD-009 vocab on the backlog model (`src/backlog.rs`)

Re-executed 2026-06-11 against the re-locked design (§10). The shipped old vocab
(`depends_on`/`before`, both `Vec<String>`) → PRD-009 `needs`/`after`/`triggers`.

- **`Relationships` is one shared struct** (raw + validated) moved whole at
  `validate` (A1 held) — the rename/retype threaded both layers in a single edit.
- **`after`/`triggers` are net-new typed surface**, not a rename: `after:
  Vec<AfterEdge>` (`AfterEdge { to, #[serde(default)] rank: i32 }`), `triggers:
  Vec<Trigger>` (`Trigger { #[serde(default)] globs, #[serde(default)] note }`).
  Both derive `Serialize` (Relationships does NOT) — `show_json` embeds
  `rel.after`/`rel.triggers` directly through `json!`, no hand-built objects.
- **Render split**: the four string axes (slices/specs/drift/needs) stay in the
  one `[(label,&refs)]` loop; `after`/`triggers` carry payload → bespoke blocks.
  Format (impl latitude): `after: B (rank 2), C` (rank suffix only when `!= 0`);
  `triggers: [g1, g2] note` (globs bracketed, note trails when non-empty).
- **A4 held — no data loss on rename.** No `deny_unknown_fields`, so the
  renamed-away `depends_on`/`before` keys default `[]`; on-disk only `risk/002`
  carried seeded-empty values. No populated old-vocab edge found (A4 STOP clear).
- **PHASE-02's `backlog_order.rs` was untouched and stays green** — it owns its
  own representation, doesn't read `Relationships`' private fields. Its corrective
  re-execution (old `depends_on` semantics → `after`) is the NEXT phase.
- VT-1 closed by: virgin round-trip (all three default `[]`),
  `after_edge_round_trips_with_optional_rank` (bare `{to}`→rank 0),
  `trigger_round_trips_with_optional_note` (`{globs}`→note ""), and
  `backlog_show_renders_all_three_item_axes` (table + JSON).

## PHASE-02 — pure cordage adapter (`src/backlog_order.rs`)

> **CORRECTED & RE-EXECUTED 2026-06-11 to the re-locked PRD-009 vocab (D10).** The
> first shipping (OLD vocab — `depends_on`/`before`, forward `before` edge `A→B`,
> `EdgeAttrs(0,0)` on every edge) is preserved in git @ the pre-correction commit;
> this section now documents the corrective re-exec. The R-C null result at the
> foot still holds (feeds PHASE-04 / OQ-B).

The vocabulary half of cordage: projects `OrderInput`, builds two overlays
(`needs` Reject+Unbounded / `after` Evict+Unbounded) + one `OrderSpec`
`[Along(needs), Along(after)]`, reads order + provenance back out as `ItemId` /
`Override`. No sort of its own (I1); pure, disk-free, `BacklogItem`-free.

- **`build` returns `anyhow::Result`, not an infallible panic.** Design A2 framed a
  cordage `BuildError` as an adapter bug to `expect`/`unreachable` on. The repo lint
  posture denies `expect_used`/`unwrap_used`/`panic`/`unreachable` in lib code, so
  the lint-clean expression is a `map_err` propagate to the boundary — same intent
  (no recoverable path, never matched for recovery), surfaced as an `anyhow` error.
- **Edge direction is now UNIFORM B→A (the load-bearing re-lock change).**
  `A.needs=[B]` ⇒ cordage edge **B→A** (`EdgeAttrs::new(0,0)`, hard edges never
  evict); `A.after=[{to=B, rank}]` ⇒ cordage edge **B→A** too (was the old forward
  `before` A→B). One flip at ingest for both relations: ingest `src` = the resolved
  predecessor (`to`/`dep`), `dst` = the authoring item. Both layers `Along`,
  uniform src-before-dst — no per-overlay direction reasoning.
- **Genuine `(rank, age)` eviction on `after`.** Each `after` edge carries
  `EdgeAttrs::new(rank, age)`: `rank: i32` the authored per-edge rank, `age: u64` =
  the edge's INDEX in that item's `after` array (`u64::try_from(idx)`, never `as`).
  cordage's eviction key is `(rank, age, src, dst)` ascending (`resolve.rs:38`) —
  lowest evicted first, so a higher-`rank` edge SURVIVES and, at equal rank, the
  lower-`age` (earlier array position) edge is dropped. Retires the old A4
  `(0,0)`-on-every-edge `(src,dst)` stand-in. Both halves of the key are unit-proven
  (`lower_rank…` for rank, `lower_age_after_edge…` for age — the age test was
  verified to FAIL if `age` were constant, so it genuinely discriminates the key).
- **`Override` orientation is uniform across all reasons** (review fix). `from`
  should have preceded `to`; it didn't, because `reason`. Evicted edges read
  `src→from`, `dst→to` (already B→A); the `Dangling` drops were corrected to match
  (`from` = the missing predecessor, `to` = the dependent) — the original corrective
  draft left dangling in the un-flipped authored orientation, inconsistent with the
  evicted paths and its own doc comment (caught by the codex adversarial pass; the
  flip's seam is exactly where `mem.pattern.review.interaction-bugs-hide-between-
  sound-parts` predicts a bug).
- **Node allocation = the tier-2..4 fallback.** Nodes minted in `(exposure desc,
  created asc, canonical-id asc)` order; cordage's monotonic `NodeId` then carries
  the fallback wherever no overlay edge constrains a pair. `ItemId: Ord` is
  `(prefix, id)` — the canonical-id ascending key, alloc-free.
- **`exposure` lives in `backlog.rs` as `exposure(Option<&RiskFacet>) -> u8`** — one
  fn, nested `weight` map (Low=1…Critical=4), `0` baseline for non-risk and
  part-assessed risk alike. This is the OQ-A split: derivation by the data,
  vocabulary in `backlog_order.rs`. Shipped in PHASE-01 (`backlog.rs:499`); the
  corrective re-exec did not re-touch it. PHASE-03's `project` calls it.
- **Whole leaf is dead in the non-test build** (CLI consumer is PHASE-03). One
  module-level `#![cfg_attr(not(test), expect(dead_code, …))]` — the cfg-test-scoped
  self-clearing pattern (`mem.pattern.lint.dead-code-expect-vs-cfg-test`); an
  unconditional `expect` would fire unfulfilled under `cargo test`.
- **Deferred (codex finding, pre-existing, NOT a corrective-delta regression):**
  duplicate `ItemId`s in `&[OrderInput]` silently corrupt the bimap (`by_item`
  overwrite while `by_node` retains both). Design §5.4 scopes the input to "one row
  per non-terminal item" (PHASE-03's `project` guarantees it), so it cannot arise in
  production — but the pure adapter doesn't fail loud on a precondition violation.
  Captured as a backlog item rather than expanding this corrective phase.
- **R-C bend (OQ-B, provisional null).** First real consumer needed **no** cordage
  API change — builder/overlay/edge/order_spec + provenance cycles/evictions
  sufficed (now also the genuine `(rank, age)` eviction key, which existed as-is).
  PHASE-04 confirms and records the budget closure.

Surface widenings in `backlog.rs`: `ItemKind::canonical_id` and `ItemKind::prefix`
→ `pub(crate)` (single-source reuse by `ItemId`, not a copy) — landed PHASE-01.

## PHASE-04 — leaf invariant pinned + R-C interface budget CLOSED (null)

Verify + harvest phase, zero production code. The cordage reserved budget is
closed, not dangling.

- **VT-9 leaf invariant — HELD end-to-end.** Of the five SL-039 commits
  (`41a0279`, `8305613`, `729529c`, `7af085e`, `12fb716`), **zero** touch
  `crates/cordage/**` (`git show --stat <c> -- crates/cordage/` → 0 files each;
  last cordage touch was SL-036/SL-038). `cargo tree -p cordage` → `cordage
  v0.1.0` alone at root, no doctrine in its subtree — cordage stayed a
  dependency-free pure leaf. The one `src/backlog_order.rs` delta this slice
  (`12fb716`, 17 lines) was removal of the now-unfulfilled self-clearing
  `#![cfg_attr(not(test), expect(dead_code))]` — that file is the **adapter** in
  the doctrine crate, NOT the cordage leaf. EX-1 holds.
- **VT-10 token-absence audit still green** (`no_pub_crate_signature_leaks_a_
  cordage_id`, a PHASE-02 deliverable, not re-authored): no `pub(crate)` signature
  leaks an opaque cordage id. Opaque ids never escape the adapter (§10 E4).
- **R-C interface finding (OQ-B / objective 5) — NULL result, budget CLOSED.**
  First real consumer (PHASE-03's CLI `backlog needs`/`after`/`order` + `project`)
  drove the **entire** cordage public surface — `GraphBuilder`/`Graph`/`OrderSpec`/
  `OverlayConfig`/`EdgeAttrs`/`EvictReason`/`CyclePolicy`/`Direction`/`Arity`/
  `OrderLayer`/`NodeId`/`OverlayId` plus the provenance cycles/evictions and the
  genuine `(rank, age, src, dst)` eviction key — with **zero cordage API change**
  (EN-2 held all three phases). No bend was needed; no latent ergonomic friction
  the shell silently absorbed. The budget reserved to discover a real-use API bend
  is spent and returned empty. Recorded durably as
  `mem.system.engine.cordage-rc-budget-closed-null`.
