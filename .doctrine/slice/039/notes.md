# SL-039 — implementation notes

Durable findings harvested from the disposable phase sheets. The runtime sheets
under `.doctrine/state/` are `rm -rf`-able; what must survive lives here.

## PHASE-02 — pure cordage adapter (`src/backlog_order.rs`)

> **SUPERSEDED 2026-06-11 by the design RE-LOCK (D10 reconcile, design.md §10).**
> This section documents the FIRST shipping of PHASE-02 against the OLD vocab —
> `depends_on`/`before`, the forward-pointing `before` edge (`A.before=[B]` ⇒
> `A→B`, *not* flipped), and `EdgeAttrs(0,0)` on every edge. The re-locked design
> renames to `needs`/`after`, flips `after` to point backward (`A.after=[{to=B}]`
> ⇒ `B→A`, uniform with `needs`), and carries genuine `EdgeAttrs(rank, age)`
> eviction on the soft overlay. The corrective PHASE-02 re-execution replaces the
> direction/EdgeAttrs claims below; supersede this section then (not yet — no code
> until the plan is re-approved). **The R-C null result at the foot of this
> section still holds and is NOT superseded** — it feeds PHASE-04 / OQ-B.

The vocabulary half of cordage: projects `OrderInput`, builds two overlays
(`depends_on` Reject+Unbounded / `before` Evict+Unbounded) + one `OrderSpec`
`[Along(dep), Along(before)]`, reads order + provenance back out as `ItemId` /
`Override`. No sort of its own (I1); pure, disk-free, `BacklogItem`-free.

- **`build` returns `anyhow::Result`, not an infallible panic.** Design A2 framed a
  cordage `BuildError` as an adapter bug to `expect`/`unreachable` on. The repo lint
  posture denies `expect_used`/`unwrap_used`/`panic`/`unreachable` in lib code, so
  the lint-clean expression is a `map_err` propagate to the boundary — same intent
  (no recoverable path, never matched for recovery), surfaced as an `anyhow` error.
- **Edge direction (D4).** `A.depends_on=[B]` ⇒ cordage edge **B→A** on the dep
  overlay (the single flip at ingest); `A.before=[B]` ⇒ edge **A→B** on the before
  overlay (already src-before-dst). Both layers `Along`; `EdgeAttrs::new(0,0)`
  (durability unused this slice).
- **Node allocation = the tier-2..4 fallback.** Nodes minted in `(exposure desc,
  created asc, canonical-id asc)` order; cordage's monotonic `NodeId` then carries
  the fallback wherever no overlay edge constrains a pair. `ItemId: Ord` is
  `(prefix, id)` — the canonical-id ascending key, alloc-free.
- **`exposure` lives in `backlog.rs` as `exposure(Option<&RiskFacet>) -> u8`** — one
  fn, nested `weight` map (Low=1…Critical=4), `0` baseline for non-risk and
  part-assessed risk alike. This is the OQ-A split: derivation by the data,
  vocabulary in `backlog_order.rs`. PHASE-03's `project` calls it.
- **Whole leaf is dead in the non-test build** (CLI consumer is PHASE-03). One
  module-level `#![cfg_attr(not(test), expect(dead_code, …))]` — the cfg-test-scoped
  self-clearing pattern (`mem.pattern.lint.dead-code-expect-vs-cfg-test`); an
  unconditional `expect` would fire unfulfilled under `cargo test`.
- **R-C bend (OQ-B, provisional null).** First real consumer needed **no** cordage
  API change — builder/overlay/edge/order_spec + provenance cycles/evictions
  sufficed. PHASE-04 confirms and records the budget closure.

Surface widenings in `backlog.rs`: `ItemKind::canonical_id` and `ItemKind::prefix`
→ `pub(crate)` (single-source reuse by `ItemId`, not a copy).
