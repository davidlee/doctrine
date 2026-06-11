# SL-038 — implementation notes: cordage scale harness findings

Durable findings note for the cordage scale-cliff harness. Consolidates the four
measured cliffs, the exact commands that reproduce each, the in-target verdict,
SPEC-001 H1's honest position, RSK-004's first-measured-here provenance, and the
OQ-2 allocation gap. Cites the filed fixes (RSK-002/003/004 + slice §Follow-Ups);
does not re-file.

All numbers are debug-build, jail target, measured 2026-06-11. They trace to the
PHASE-01 example probe (`examples/scale_harness.rs`) and the PHASE-02 durable reds
(`tests/scale_cliffs.rs`); no value is invented here.

## 1. The four cliffs (measurements + reproduce-commands)

Each cliff has two reproduce paths: the durable `#[ignore]`d red in
`tests/scale_cliffs.rs` (the committed proof), and the `examples/scale_harness.rs`
CSV probe (the exploratory instrument). Run the whole red suite with:

```
cargo test -p cordage -- --ignored --nocapture --test-threads=1
```

| Cliff | Risk | Class | Measured | Red test | Example probe |
|---|---|---|---|---|---|
| explain exponential | RSK-002 | exact `2^layers` | path count `== 1<<18` = **262_144** at L=18 (verified L=4→16, L=12→4096, no extrapolation) | `--exact explain_path_count_is_exponential_in_diamond_depth` | `--cliff explain --layers 18` |
| build overflow | RSK-003 (primary) | stack-depth crash | **`rc 134`** (SIGABRT, "stack overflow, aborting") at chain depth **80_000** | `--exact deep_chain_overflows_inside_target_scale` | `--cliff overflow --n 80000` |
| eviction quadratic | RSK-003 (secondary) | super-quadratic | ratio **~18.5–18.7×** (4× edges / 2× nodes), `dense_evict(50,50)` vs `(100,100)` (PHASE-01 pin 18.5×, PHASE-02 18.7×, audit re-run 18.5×) | `--exact eviction_fixpoint_scales_superlinearly` | `--cliff quadratic --n {50,100}` |
| evaluate quadratic | RSK-004 | clean quadratic O(V²) | ratio **~4.2–4.3×** (2× nodes), `deep_chain(2000)` vs `(4000)` (PHASE-01 pin 4.25×, PHASE-02 4.3×, audit re-run 4.2×; ~4× = quadratic) | `--exact evaluate_scales_quadratically_in_node_count` | `--cliff evaluate --n {2000,4000}` |

Per-cliff mechanism (source anchors carried from the slice scope):

- **explain** — `explain()` enumerates all predecessor paths; `extend_chains`
  (`query.rs:150/158`) clones the suffix per branch. `2^layers` growth ⇒ OOM/hang
  beyond the lattice depths measured.
- **overflow** — `Tarjan::strongconnect` (`resolve.rs:321`) and `level_of`
  (`resolve.rs:545`) recurse with depth = graph depth. Two **independent**
  overflows; `level_of` overflows on a clean acyclic chain with no cycle present.
- **eviction quadratic** — `pass2_evict` (`resolve.rs:198`) / `evict_layer_cycles`
  (`resolve.rs:478`) recompute a full SCC pass per evicted edge; `participates`
  (`resolve.rs:224`) rescans all components per candidate, compounding to
  O(E·(V+E)).
- **evaluate** — `evaluate()` (`query.rs:256`) runs a fresh `reachable()` BFS per
  node — O(V·(V+E)); over a sparse deep spine that is O(V²). `values=1` every run
  confirms the cost is the *unconditional* per-node BFS, not the fold.

## 2. In-target verdict

The discovery question — *are these cliffs reachable inside the revised ~tens-of-
thousands-of-nodes target?* — is answered **yes, for all four**:

- overflow crashes at chain depth ~80k — inside tens-of-thousands, not beyond;
- eviction is super-quadratic from the low hundreds of nodes;
- explain is `2^layers` — a few dozen diamond layers exhaust memory;
- evaluate is O(V²) per-node BFS over a deep spine.

None requires going past the target scale to manifest. The four cliffs are durable,
reproducible, and committed as red.

## 3. SPEC-001 H1 — honest position

H1 (recompute-per-query) was revised 2026-06-11 (`54bd3f4`,
`spec(SPEC-001): revise H1 scale target to ~tens of thousands`) from a "small corpus
(tens–hundreds)" premise to the real ~tens-of-thousands target. Against that target:

- H1's recompute-per-query claim **holds only after Fix A / Fix B / Fix D** — until
  the overflow, eviction-quadratic, and evaluate-quadratic cliffs are fixed, build +
  evaluate are not O(V+E) at target scale.
- **explain needs Fix C before any deep-lattice consumer** — its `2^layers` blow-up
  is not a scale constant, it is exponential; no recompute budget survives it.

The trailing **SPEC-001 H1 wording reconcile** is gated on the fixes landing and is
**out of scope for this slice** (SL-038 is measure-and-red only). Tracked in the
slice §Follow-Ups.

## 4. Provenance (RSK-004 is first-measured-here)

Provenance is load-bearing — the numbers do not all share an origin:

- **RSK-002 (explain) and RSK-003 (overflow + eviction quadratic)** numbers are the
  **deleted post-close probe's**, *re-confirmed* by this harness. The probe left
  these numbers in the risk records but no committed artifact; this slice reproduces
  them and pins them in a durable red.
- **RSK-004 (evaluate quadratic) is first measured here.** The deleted probe
  **never ran** the evaluate cliff — RSK-004 was filed *analytically*, after the
  probe, from a read of `query.rs:256` (folded in at design D5). This harness is its
  **sole empirical source**, not a reproducer of a prior run. PHASE-01 obtained the
  first measurement (4.25×), PHASE-02 pinned it as a red (4.3×).

## 5. Fixes filed (cite, don't re-file) + OQ-2 gap

The four fixes already have durable homes — this note **cites** them, it does not
open new backlog items:

- **Fix A** (iterative `strongconnect` + `level_of`) → **RSK-003** `controls`
  (primary overflow).
- **Fix B** (one-pass / incremental-SCC eviction, drop the `participates` rescan) →
  **RSK-003** `controls` (secondary quadratic — RSK-003 covers *both*).
- **Fix C** (explain returns a predecessor sub-DAG, enumerate on demand) →
  **RSK-002**.
- **Fix D** (single reverse-topo fold for `evaluate()`) → **RSK-004**.

All four are also enumerated in **slice-038.md §Follow-Ups** with their TDD framing
(each greens against one of this slice's reds). All three risks are **open**.

**OQ-2 — allocation gap.** This harness is **wall-clock-first**. Peak-allocation
measurement (allocator shim / external `time -v`) is **out of scope for v1** and
noted here as the standing gap. The deleted probe's OOM observations already bound
the explain blow-up *qualitatively*; a quantitative peak-alloc number is deferred.

## 6. Reconcile

- Harness untouched in PHASE-03 (docs-only); `just check` green.
- Every measurement in §1 traces to a prior sheet and a reproduce-command (storage
  rule: no invented derived data).
- Slice is audit-ready. VH-1 (user accept) is the remaining gate.
