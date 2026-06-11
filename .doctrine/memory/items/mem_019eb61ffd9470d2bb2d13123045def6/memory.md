# cordage R-C interface budget closed (null result, no bend needed)

SL-039 reserved an R-C budget (design OQ-B / objective 5) to discover what cordage
API bend a real first consumer would demand. The budget is **spent and CLOSED
empty** — no bend was needed.

PHASE-03's CLI (`backlog needs`/`after`/`order`) + `project` (`src/backlog_order.rs`,
the consumer-half adapter) drove the **entire** cordage public surface —
`GraphBuilder`/`Graph`/`OrderSpec`/`OverlayConfig`/`EdgeAttrs`/`EvictReason`/
`CyclePolicy`/`Direction`/`Arity`/`OrderLayer`/`NodeId`/`OverlayId`, plus provenance
cycles/evictions and the genuine `(rank, age, src, dst)` eviction key — with **zero
cordage edits across all three phases** (EN-2 held). No latent ergonomic friction
the shell silently absorbed either.

Implications for a future cordage consumer:
- The leaf invariant held end-to-end (VT-9): zero `crates/cordage/**` diff this
  slice; `cargo tree -p cordage` → `cordage v0.1.0` alone, no doctrine in subtree.
- The existing public surface is sufficient for a backlog-style overlay-ordering
  consumer. Don't expect to need an API change for a similar use; if you do, that
  is a NEW finding, not a re-open of this one.
- Opaque cordage ids never escape the adapter's `pub(crate)` signatures (§10 E4,
  pinned by VT-10 `no_pub_crate_signature_leaks_a_cordage_id`).

Related: [[mem.system.spec.composition-seam]]; the deferred dup-`ItemId` bimap
fragility (filed as a backlog item at SL-039 close).
