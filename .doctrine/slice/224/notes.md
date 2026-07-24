# Notes SL-224: Honest dispatch refusals

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-24 · reconcile-complete (pre-close) · edge 141e5481

### Produced
- RV-296 — reconciliation ledger (5 findings, all verified terminal; no blocker).
  Synthesis + Reconciliation Brief + Reconciliation Outcome carry the durable record.
- Impl on review/224 (impl bundle, tip e414a7ef28a6) / dispatch/224 (canonical
  authored state, tip 444de78441f1).
- design.md reconciled to impl (§5.2/F4, §5.2 MCP, §5.5/A2 — RV-296 F-1/F-2/F-3),
  commit 141e5481. Audit ledger commit 84d31d86.

### Learned
- Impl faithful to design §5.1–§5.5; both behaviour-preservation gates
  (`check_vt_shape`, `classify_import` verdict) byte-for-byte identical to main.
- Five design-vs-impl gaps, all benign (F-1..F-5 on RV-296): three are correct
  refinements the design prose trailed (now reconciled); one discretionary seam
  (`plan_check_report`); one topology reporting artifact (IMP-228).
- Phase sheets (funnel arm) carry no durable Risks/Decisions/Findings — nothing to
  lift beyond the ledger.

### Open (→ /close)
- **Integrate the code onto trunk.** review/224 impl bundle + dispatch/224 authored
  selector state (incl. the `mod.rs` design-target selector) must land on main.
  Follow `dispatch status --slice SL-224` / `dispatch candidate status --slice
  SL-224` guidance — trunk moved ahead of the prepared base (refresh-base + re-prepare
  likely needed). NO pre-integrate `slice selector` edit on edge (forks slice-224.toml).
- **Close-gate freshness.** Run `doctrine check gate` at close on the landing tree
  (build-before-validate → fresh binary); DOCTRINE_BIN → coord build if a non-Rust
  fall-through.
- **Do NOT read as delivery gaps** (RV-296 close guardrails): edge `verify-vt` FAIL ×4
  and edge `conformance` `mod.rs` undeclared — pre-integration coord-topology
  staleness (IMP-228); tests green + selector declared on the delivered surface.

## Close (2026-07-24)

**Landed.** SL-224 `done` (2/2). Code + `mod.rs` selector + reconciled design.md
on `main`=`edge`=`681d9f1c`; journal trunk row `verified` (payload `367d7a0e`).
Post-landing on the primary tree: `conformance` 6/6 (mod.rs conformant), `verify-vt`
4/4 PASS — confirming the RV-296 F-3/F-5 edge over-reports were exactly staleness.
IMP-256 resolved `fixed`.

**Route taken — split-lineage-reconcile-on-edge** (`mem.pattern.dispatch.close-split-lineage-reconcile-on-edge`,
2nd application after SL-220). Three divergent lineages at close: code on `review/224`
(9 commits behind trunk), reconciled truth (design.md/RV-296/harvest) on `edge`,
canonical selector (`mod.rs`) on `dispatch/224`. `dispatch status`'s "refresh-base +
re-prepare" hint would have projected code-only and stranded the reconcile — the naive
path is silently wrong here. Fix: unite `edge ⊕ review/224` on a scratch worktree
(conflict-free — the slice-224.toml divergence was *disjoint*: edge touched `status`,
the bundle added the selector → clean auto-merge), gate, FF main → M, no-op
`close_target` candidate → admit → `sync --integrate --trunk main` (records the row),
reunite edge via merge (edge had advanced with SL-226/204/227).
