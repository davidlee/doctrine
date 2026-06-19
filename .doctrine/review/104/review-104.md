# Review RV-104 — reconciliation of SL-112

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed:** candidate `cand-112-review-001` (3-way merge of
`refs/heads/review/112` onto `refs/heads/main` at db948c4).

**Lines of attack.** This audit checks PHASE-01 (classify-first) and PHASE-02
(gate) against design.md and plan.toml. PHASE-03 (ADR-001 governance amendment)
is pending by design — it is the close co-requisite delegated to `/reconcile`
per the audit's reconciliation brief.

**Probes:**
1. Gate green? — `cargo test --test architecture_layering` (17 passed / 0 failed / 1 ignored)
2. Clippy clean? — zero warnings
3. `just gate` runs the test? — yes (`test-all` includes root package)
4. Authored canon complete? — `layering.toml` [tiers] section, accepted violations,
   tangle baseline
5. Implementation matches design contracts? — type shapes, four assertions,
   sub-classification, pre-flight probe
6. Bite-tests cover all violation types? — UpwardEdge, MixedUmbrella, TangleGrew,
   StaleAccepted, Unclassified, StaleEntry

**Candidate surface diff:** 3 files, +1521 lines — `.doctrine/adr/001/layering.toml`,
`Cargo.toml` (+`syn` dev-dep), `tests/architecture_layering.rs` (extractor + gate).

## Synthesis

SL-112 ships a working layering enforcement gate with honest baselines and
proven teeth. The implementation is faithful to the locked design: the four
assertions (completeness, cross-tier direction, MixedUmbrella forcing, tangle
ratchet) are all present and correctly implemented, the pure `check()` core is
unit-tested against synthetic inputs covering every violation type, and the
real-graph gate passes against the production graph under the authored baselines
(17 tests green, clippy zero-warning).

**What was right.** The classify-first spike (PHASE-01) paid off — the
`discover_units()` + `extract_edges()` extractor found 67 production units and
359 edges, and the subsequent classification produced a meaningful engine core
(18 units, clean DAG) with a small accepted-violation baseline (10 edges). The
MixedUmbrella forcing function correctly flags `catalog` and `priority` as mixed
umbrellas requiring sub-classification — the design's round-2 adversarial finding
(C2) was genuinely fixed, not papered over. The per-tier tangle ratchet uses
Tarjan SCC to count same-tier edges inside non-trivial SCCs, catching the
edge-inside-blob case the rejected `Σ(SCC−1)` metric missed.

**What was found.** Three minor/nit findings, all dispositioned:
- **F-1 (minor, fix-now):** `relation_graph` is classified as COMMAND in a
  prose comment but omitted from the `[tiers]` table — the gate works around it
  with a hard-coded fallback in `load_layering()`. Fix the authored canon, remove
  the fallback.
- **F-2 (nit, aligned):** `MixedUmbrella` variant drops the `file` field from
  the design's type contract — no functional harm; the module name suffices.
- **F-3 (minor, aligned):** `main` is exempted from the completeness check
  (binary entrypoint, not an architectural module) — reasonable, not in design
  prose.

**Standing risks.** The manual `[[accepted_violation]]` parser in
`load_layering()` is fragile to TOML reformatting (it expects
`from = "A"; to = "B"` inline syntax). The boundary-stability caveat (design R8)
remains: folding/splitting units shifts the tangle count, but the partition is
reviewed canon (`layering.toml`, REV-routed) — not a silent dodge path. The
gate's literal `crate::` path scope means macro/re-export laundering can evade it
(design F-4) — no present breach, review-covered.

**PHASE-03 is the close co-requisite.** ADR-001 currently *rejects* this test,
so the gate ships under a live governance contradiction until amended. The
amendment is routed through the reconciliation brief below.

## Reconciliation Brief

### Per-slice (direct edit)

- **layering.toml [tiers]:** Add `relation_graph = "command"` entry (F-1).
  Remove the hard-coded `relation_graph` fallback from `load_layering()` and its
  FIXME comment.

### Governance/spec (REV)

- **ADR-001 amendment (PHASE-03 / design EX-3 / D5):** Overturn ADR-001's
  rejection of the homegrown module-graph test. Rationale: the cycles arrived;
  `syn` removes the brittleness that grounded the rejection. Record the fitness
  gate as now-enforcement. Replace the stale per-module prose tier table with
  tier *definitions* plus a pointer to `.doctrine/adr/001/layering.toml` as the
  authoritative assignment. State: rule 1 hard-gated (literal `crate::` path
  edges), rule 2 a non-increasing cyclic-edge ratchet with the command tangle
  openly unmet-and-tracked, rule 3 deferred. Reclassify `input` as engine.
  Authored as a REV per ADR-013 (governance changes route through a Revision).
- **ADR-001 Verification section:** Amend to record the gate test as the
  enforcement mechanism (replacing the rejected stance).
