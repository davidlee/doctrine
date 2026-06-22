# Review RV-138 — reconciliation of SL-133

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit (self-audit) of SL-133 — multi-dimensional priority scoring —
after PHASE-01..05 + the RV-137 external pass. **Surface reviewed:** the dispatch
candidate `cand-133-review-001` (impl bundle `review/133` projected onto `main`),
admitted at `dd099115`. PHASE-05 source landed via the claude/Opus dispatch arm
(`52bfc0ce` on `dispatch/133`) after the pi arm aborted mid-phase twice.

**Lines of attack / invariants pinned:**
- **I3 (mint vs display).** Mint orders by `base.total()` only — no `score`
  feedback into graph construction; survey/next sort by `score` post-build.
- **`next` frontier sort (RV-132 F-3).** Induced-frontier Kahn over the actionable
  set, surviving-seq precedence, ready-set `(score desc, id asc)` — NOT cordage
  `order_key` (Level-before-NodeId demotes score-promoted successors).
- **EX-2 no parallel impl.** Old `consequence:u32` field + pre-pass +
  `counts_toward_consequence` gone; no dual path.
- **EX-5 no surviving suppression.** Every `expect(dead_code, "consumed PHASE-05")`
  removed (live consumers now).
- **I2 NaN/∞-free.** `is_finite`-sanitized dims/score.
- **Canon coherence.** ADR-015 / design §5.1-§5.4 vs the landed code.

**Evidence:** on the admitted candidate — `cargo clippy` zero warnings;
`cargo test` (root package) **2766 passed / 0 failed**; VT-5 (leverage reorder),
VT-7 (mint=base / survey=score / next frontier sort: Y-fixture, same-chain,
evicted-seq), VA-1 (explain --json breakdown) all green.

**Known gaps probed:** (1) ADR-015 `dep_coeff` domain wording lags code (RV-137 F-3,
governance → reconcile); (2) F-4 value_dim-formula doc edits applied on the dispatch
branch must reconcile onto canonical canon; (3) web/map TS cutover is unverifiable
offline (no `node_modules` in the jail); (4) the design-§5.4 seq-eviction subtraction
is empirically a no-op (cordage `in_edges` already excludes evicted edges).

## Synthesis

SL-133 delivered multi-dimensional priority scoring end-to-end: a pure `base_score`
(value_dim/risk_dim), a recursive `needs`-leverage DP over the dep-overlay
condensation, one-hop `ref`-optionality, and the atomic surface cutover from the old
inbound-count `consequence:u32` to a weighted `score:f64` across survey/next/explain/
inspect (+ the web map consumers). The slice's whole point — that a blocker gating
one high-value item should outrank one gating five worthless ideas — is proven by
VT-5; the no-feedback discipline (mint=base, display=score) by VT-7.

The correctness-critical engine was hardened by an external pass (RV-137, codex/
GPT-5.5): two real DP bugs (F-1 component-topo order under seq perturbation; F-2
external-dependent double-count) were fixed by rebuilding the leverage DP over an
explicit condensation DAG in reverse-topo order with per-component dependent
dedup — landed before the surface cutover. The cutover itself (PHASE-05) went
through the dispatch funnel: the pi/deepseek arm aborted mid-phase twice (embed-loop,
then a backend turn cap), so the phase was driven on the claude/Opus arm; the worker
produced a clean single-commit delta and the orchestrator completed the out-of-scope
web/map field rename.

**Closure story:** all five PHASE-05 EX criteria met; old path fully removed (no
parallel impl); no `expect(dead_code)` survives; full suite green (2766/0) and clippy
clean on the admitted candidate (`dd099115`, impl bundle on main).

**Standing risks / consciously accepted tradeoffs:**
- Web/map TS is compiler-unverified in the offline jail (F-3, tolerated) — mechanical,
  type-aligned; CI compiles + rebuilds the dist embed.
- Leverage double-counts a reconvergent diamond's shared leaf (design I5, accepted as
  the honest "downstream cone value"; path-dedup is the escalation, not this slice).
- The seq-eviction subtraction is a defensive no-op today (F-4) — retained against a
  future change in cordage's edge enumeration.

## Reconciliation Brief

### Per-slice (direct edit)
- `design.md` §5.1: the `value_dim` formula gains the `coefficients.value` factor —
  already edited on `dispatch/133` @3ddeb307 (F-2); lands on `main` via the
  candidate→main integrate at `/close`. Confirm it survives the integrate.
- `design.md` §5.2/§7: align the `dep_coeff` domain wording to `[0,1]` with `0` the
  disable sentinel (F-1), matching code.

### Governance/spec (REV)
- `ADR-015` (F-1 + F-2 → one REV modify): (a) ratify the `dep_coeff` domain as
  `[0,1]` with `0` (and any clamped `≤0`) the explicit DISABLE sentinel — supersedes
  the `(0,1]` wording; (b) add the `coefficients.value` factor to the `value_dim`
  formula. No executable change; code is the coherent reading.

### No action (recorded for completeness)
- F-3 (web/map TS verification gap) — tolerated, environmental.
- F-4 (seq-eviction subtraction no-op) — aligned, safeguard retained; captured as
  `mem.fact.cordage.in-edges-excludes-evicted`.
- F-5 (conformance pass) — aligned.

## Reconciliation Outcome

### Direct edits applied (per-slice, on edge)
- `design.md` §5.1: `value_dim` formula gains `coefficients.value` (RV-138 F-2).
- `design.md` §5.2 (line ~112) + the `ConsequenceCoeffs` doc comment (~174) + §7
  asymmetry mentions (498/510/514/648): `dep_coeff` domain `(0,1]` → `[0,1]` with `0`
  (and any clamped `≤0`) the explicit disable sentinel (RV-138 F-1).

### REVs completed
- **REV-008** (`reconcile-sl-133`): done — `ADR-015` amended (one `modify` row,
  manually landed on edge): (a) `dep_coeff` domain `(0,1]` → `[0,1]` with `0` the
  disable sentinel (covers F-1); (b) `value_dim` formula gains `coefficients.value`
  (covers F-2). Rationale + before/after in `revision-008.md`. No executable change.

### Tolerated / aligned (no write)
- F-3 tolerated (offline-jail TS verification gap); F-4 aligned (defensive no-op
  safeguard, memory recorded); F-5 aligned (conformance pass).

### Topology note for /close
The candidate (→ `main` at integrate) already carries the `value_dim` doc edits
(applied on `dispatch/133` @3ddeb307) — **identical content** to the edge edits here,
so the post-integrate `edge`↔`main` merge is conflict-free. The `dep_coeff` domain
edits exist **only on edge**; `/close` must ensure `main` ends with the **union**
(edge is the superset of the authored truth). All reconciled authored truth is
complete on `edge`.
