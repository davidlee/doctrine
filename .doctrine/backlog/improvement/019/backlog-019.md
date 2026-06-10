# IMP-019: cordage golden_net proves determinism but not value-correctness: no independent value oracle

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found in the SL-036 post-close code review (codex GPT-5.5 + Opus, independent
agreement). The determinism *machinery* is sound; the *proof* under-checks.

**What golden_net.rs actually proves.** The independent oracle (`oracle_sccs`,
`golden_net.rs:289`; topo witness `:396`) is genuinely independent — mutual-
reachability closure, not Tarjan — and cross-checks **SCC node-sets** and
**topo edge-respect**. That part is sound.

**The gap.** `order_key` *level values*, `Channel` *folded values*, eviction
*selection*, and *contributor sets* are checked **only** for permutation-
invariance and build-twice equality — i.e. self-consistency. No independent
oracle confirms the computed values are *correct*. A deterministic-but-wrong
level/fold/eviction would pass every golden_net test.

Secondary gaps:
- Permutation covers **edge-insertion order only** (`build_in_order`, `:64`);
  node-mint, overlay-allocation, and seed-map order are never permuted — so the
  "any input order" claim is broader than the coverage.
- Build-twice fixtures don't exercise multi-layer `U` composition, union-cycle
  eviction, degraded-SCC taint, or `Explanation` equality on those cases.
- Every fixture uses `Direction::Along` — `Against` re-map untested (ties
  [[RSK-001]]).

**Improvement:** add an independent value oracle for at least one non-trivial
fixture (hand-computed expected levels/folds/eviction), widen permutation to the
other order-insensitive authored inputs, and add rebuild-equality fixtures for
union-cycle + degraded-order cases. Either that, or narrow the claim wording so
it states "insertion-order insensitivity", not "proven correct".
