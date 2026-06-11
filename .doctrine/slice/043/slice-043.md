# cordage scale & robustness hardening

## Context

SL-036 shipped the `cordage` graph core (SPEC-001 D1 carve-out) and closed at
H1/H2 scale (hundreds of nodes; all VT fixtures ≤5 nodes). A post-close perf
review on 2026-06-11 (codex GPT-5.5 + Opus, independent agreement) raised the real
scale target to **tens of thousands** and filed four open risks. At that target the
core has both **correctness cliffs** (stack overflow → panic) and **complexity
cliffs** (exponential / quadratic blowup). None are exposed by the current suite —
every VT fixture is shallow — so they are silent until the first consumer feeds a
deep/large real graph (the adapter/policy slices descending from SPEC-001).

This slice retires that risk cluster as one coherent hardening pass. The four
risks share a substrate: they live in the same two files (`resolve.rs`,
`query.rs`), they are all invisible at fixture scale, and they all require the
**same missing test scaffolding** — synthetic graph generators (deep chain,
diamond lattice, large fan-out) plus scale/cliff probes. Building that harness
once and pointing it at all four is the cohesion argument.

Governing canon (prose links, not edges): SPEC-001 §Decisions, Appendix B
(forbidden-core list — the fixes must stay product-neutral); ADR-001 (cordage is a
leaf); ADR-004 (reverse edges derived, never stored). Pure/imperative split holds —
the iterative rewrites stay in the pure layer (explicit stack, no new I/O).

Empirical confirmations already on the risk cards (Opus probes, since deleted):
RSK-002 — 16.7M chains at 24 diamond layers in 1.1s, 2^layers growth, OOM beyond.
RSK-003 — both recursive sites SIGABRT (rc 134) at chain depth ~80k, **inside**
target scale; secondary quadratic eviction measured ~17× for 4× edges.

## Scope & Objectives

Harden the cordage core to the tens-of-thousands target along four axes, each
retiring one risk. Fix **directions** below are the risk-card hypotheses; the
perf spike validates them before any fix lands — `/design` owns the final calls.

- **RSK-003 — recursion overflow (impact: high).** `Tarjan::strongconnect`
  (`resolve.rs:321`) and `level_of` (`resolve.rs:545`) recurse at graph depth and
  SIGABRT the 8MB stack at ~80k. Direction: mechanical explicit-stack iterative
  rewrite of both (the two overflows are independent — `level_of` blows on a clean
  acyclic chain). The behaviour-preservation gate applies: existing suites stay
  green unchanged.
- **RSK-003 secondary — eviction quadratic.** `pass2_evict` (`resolve.rs:198`) and
  `evict_layer_cycles` (`resolve.rs:478`) recompute a full Tarjan SCC pass per
  evicted edge → O(E·(V+E)). **Locked (design D4):** per-SCC localization — one
  Tarjan, then re-test only the shrinking touched component; SCC disjointness makes
  the evicted *set* provably identical (provenance sorted). A single dense SCC stays
  contract-bound superlinear (design EXC-2) — not fixable without changing the
  evicted set.
- **RSK-004 — channel-eval quadratic (impact: medium).** `evaluate()`
  (`query.rs:256`) runs a fresh full `reachable()` BFS per node → O(V·(V+E)).
  **Locked (design D3):** build-SCC condensation + reverse-topo fold, reusing the
  build-time `degraded_sccs` (no query Tarjan); cycle-safe (F47). Idempotent
  combinators O(V+E); CountDistinct set-union fold superlinear (design EXC-1).
- **RSK-002 — explain() enumeration exponential (impact: medium).** `explain()`
  enumerates every chain → 2^layers; `extend_chains` also clones the suffix per
  branch. **Locked (design D2):** return the **pure predecessor sub-DAG**
  (`paths`→`predecessors`), no witness chain — D11/F13 role-agnostic structure;
  policy enumerates on demand. No non-test consumer (output-contract change is free).
- **RSK-001 — Against-orientation re-map untested (impact: low).** `orient`
  (`resolve.rs:463`) is correct but exercised only indirectly. Add a direct VT
  (coverage-add, passes on first write); lives near `orient` in resolve.rs.

**Deliverable spine (fixes only — the harness is SL-038, `done`):** the four fixes
above, phased P1 `resolve.rs` (RSK-003 both + RSK-001) → P2 `query.rs` (RSK-004) →
P3 `query.rs`+`lib.rs` (RSK-002), each TDD red→green→refactor against SL-038's
`#[ignore]`'d reds.

## Non-Goals

- **RSK-005** (backlog_order adapter NodeId-bimap corruption) — different module
  (`src/backlog.rs`), different slice (SL-039), correctness bug not scale. Out.
- **Adapter / policy / CLI layers** of SPEC-001 — they consume cordage; not
  touched here beyond keeping the public surface intact.
- **IMP-019** (independent value oracle) and **IMP-020** (query.rs traversal
  triplication: `reachable`/`spine_path`/`extend_chains` diverged walks) — adjacent
  and *tempting*: RSK-004 and RSK-002 both rewrite walks IMP-020 wants consolidated.
  Possible fold-in, but they widen scope from "close the risk cluster" to
  "refactor the traversal core." Decision deferred to `/design`; default is out.
- **Forbidden-core violations** — no time/urgency/product vocabulary may enter the
  crate via any fix (SPEC-001 Appendix B).

## Open Questions

- **OQ-1 — RESOLVED (design D2).** RSK-002 *does* change the `explain` output
  contract, but no non-test consumer reads it → free to reshape to the sub-DAG.
- **OQ-2 — IMP-020 fold-in.** Still open; default **out** (design OQ-2). Revisit
  only if the RSK-004/RSK-002 rewrites naturally converge a shared traversal helper.
- **OQ-3 — REQ pin.** Exact REQ subset under SPEC-001 for
  `relationships.requirements` — resolve at `/plan`.
- **OQ-4 — RESOLVED.** Harness is SL-038 (`done`); phase shape is 3-by-file/concern
  (design D6), not perf-spike-first.

## Verification / Closure Intent

- SL-038's `#[ignore]`'d cliff reds are flipped/inverted to green gates (overflow
  → build-succeeds; evaluate → near-linear; explain → cone-shape; eviction → new
  many-small-cycles linear gate, `dense_evict` stays `#[ignore]` per EXC-2).
- Each risk has a **red test that fails on `main`** (overflow / measured blowup)
  and passes after the fix; cliff bounds budget for debug-build timings running
  ~10× release (recorded memory).
- All pre-existing cordage suites stay green unchanged (behaviour-preservation).
- `cargo clippy` zero warnings; `just check` clean.
- RSK-001..004 transitioned to resolved via `backlog edit` at `/close`, each citing
  the VT/fix that retired it.

## Follow-Ups

- RSK-005 stays open for SL-039's lineage / its own fix.
- IMP-019 / IMP-020 remain open unless folded at `/design`.
