# SL-101 Estimate & Value facets are not integrated into main

SL-101 ("Estimate & Value facets") closed `done`, but its delivery to `main` is
incomplete:

- `src/estimate.rs` exists on `main` but is **dead/unwired** — `#![allow(dead_code)]`
  at the top, and nothing references `estimate::` / `EstimateFacet` /
  `EstimationConfig` outside the file. `dtoml.rs` wires only `[conduct]` +
  `[verification]` — no `[estimation]`. `slice.rs` has no facet display.
- `src/value.rs` does **not exist on `main`** at all (no `mod value`). The Value
  facet module + wiring (dtoml `[value]`, slice.rs display, config example) live
  only on `dispatch/101`, `dispatch/101-phase-03`, `dispatch/101-phase-04` —
  never merged. `main` has diverged ~34 commits past their merge-base
  (`d22ab0be`); `merge-tree` conflicts on `estimate.rs`. Not cleanly mergeable.

So the dispatch candidate code is reference material, not a merge source.

**Spec side is now reconciled** (REV-002, commit 446806cb): PRD-014 renamed the
unit default to `espresso_shots` and integrated the Value facet (worth/payoff
counterpart to the Estimate's attention-burden cost; unit `magic_beans`); SPEC-020
REQ-271 + the Value stubs REQ-278/279/280 fixed/enriched. The unit rename DID land
in `src/estimate.rs` (`DEFAULT_ESTIMATION_UNIT = "espresso_shots"`).

**Still owed:** a new integration slice (SL-101 cannot be reopened — `done` is
terminal, ADR-009) that wires both facets onto current `main`, porting the
stranded `dispatch/101` code and re-verifying. Supersedes the
"reconciliation must produce a Value PRD+SPEC" half of [[mem_019edacf8807772285610b0de376fdc5]]
— that PRD/SPEC work is done; only code integration remains.
