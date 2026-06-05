# Audit SL-016: Break slice↔state cycle: extract plan types

Conformance audit (post-implementation). Reconciles the implemented PHASE-01
against `design.md`, `plan.toml`, and ADR-001. Hand-authored (no audit scaffold
yet). Evidence gathered 2026-06-05 at commit `24d6144`.

## Mode

Conformance — single-phase slice, all phases complete (1/1). Pure structural
move; the behaviour-preservation gate (unchanged green suite) is the primary
proof.

## Evidence

Gate (`just check`, post `touch tests/*.rs`): **green** — 406 unit + 3 e2e
memory + 1 e2e skills, assertions unchanged. `cargo clippy` zero warnings.
`cargo build` compiles acyclically.

| # | Check | Command | Result |
|---|---|---|---|
| E1 | `plan` is an engine-tier leaf | `grep 'use crate::' src/plan.rs` | none (deps: serde/anyhow/toml only) — **leaf** |
| E2 | cycle severed at the edge | `grep 'crate::slice' src/state.rs` | empty; `state→plan` present (`state.rs:28`) |
| E3 | no other consumer of slice's plan types | `grep -rn 'crate::slice::.*Plan' src/` | none |
| E4 | `read_plan` stays in shell, calls parse | `src/slice.rs:171,176` | `read_plan` present; `Plan::parse(&text)` |
| E5 | `parse` widened to `pub(crate)` | `src/plan.rs:44` | `pub(crate) fn parse` |
| E6 | selective test relocation (A3) | grep both files | `:711/:727`→`plan.rs`; `render…substitutes` + `…accepts_scaffold` stay in `slice.rs` |
| E7 | `PhaseStatus` clap leak untouched (D3) | `src/state.rs:17,37` | `clap::ValueEnum` still present — out of scope, retained |

## Findings & dispositions

- **F1 — cycle broken; graph acyclic.** Expected (design §5.1: `state→slice`
  removed, replaced by `state→plan`; `plan` is a leaf). Observed: E1/E2/E3.
  → **aligned.**
- **F2 — read/parse split honoured (D1).** Expected: `parse` moves, `read_plan`
  (disk) stays in slice. Observed: E4/E5 — `read_plan` in slice calls
  `plan::Plan::parse` cross-module; impurity stays in the shell, engine leaf is
  pure. → **aligned** (ADR-001 + pure/imperative split satisfied).
- **F3 — selective test relocation (A3).** Expected (design §9): only the two
  self-contained inline-TOML tests move; the two contract tests calling
  slice-private `render_plan_toml` stay. Observed: E6 exactly. → **aligned.**
- **F4 — slice.rs import narrowed to `Plan` (refinement of §5.2).** Design §5.2
  spoke of slice importing "the types"; implementation imports `crate::plan::Plan`
  **only** — `PlanPhase` has no remaining slice.rs consumer (`read_plan` returns
  `Plan`; nothing else names `PlanPhase` in slice). Importing the pair would have
  tripped the repo `unused_imports` deny. Caught at clippy, narrowed. Orphaned
  `anyhow::bail` + `serde::Deserialize` likewise removed; `Context` retained
  (`slice.rs:175,260` still use it). → **aligned** (design intent preserved; the
  exact import set is below the design's altitude, correct under the lint regime).
- **F5 — `PhaseStatus` clap leak retained (D3).** Expected: orthogonal to the
  cycle, left in place by user decision. Observed: E7 — present; the rewritten
  `state.rs` debt comment now names it as the residual, out-of-scope item.
  → **aligned** (conscious carry; not a regression). Follow-up: split the CLI
  enum from the stored value if a second consumer of `PhaseStatus` appears.
- **F6 — visibility widening (R3).** `parse` private → `pub(crate)`. Bounded to
  the crate, required for the shell/engine boundary. → **aligned.**

## Reconciliation

No divergence between code and design. No `design.md` correction needed; no
fix-now items. The single sub-altitude refinement (F4) is recorded here rather
than back-propagated — the design's "import the types" intent holds; the lint
regime determined the exact symbol set, which is execution detail.

## Closure readiness

Audit-ready. All findings **aligned**; zero fix-now, zero tolerated drift. One
named follow-up (F5: `PhaseStatus` clap split, deferred per D3). Lifecycle status
divergence (`slice list` ⚠ 1/1-vs-`proposed`) is the known transition gap —
reconcile `slice-016.toml` `status` → done at `/close`.

## Durable harvest

- Phase-sheet findings folded in (F4 import-narrowing, F5 leak carry).
- Env gotcha (stale-`CARGO_BIN_EXE` spawn-fail) already covered by
  `mem.pattern.testing.stale-cargo-bin-exe` — no new memory needed.
- No new reusable pattern surfaced; the lift-and-shift was mechanical.
