# Implementation Plan SL-133: Multi-dimensional priority scoring for survey/next

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases that flow strictly forward along the hard dependencies the design
fixes: **ratify policy → extract the leaf the inputs need → read the inputs →
land the pure scoring engine additively → cut the surfaces over and activate**.
The plan deliberately differs from design §9's provisional P1–P5 in two places
(both noted below); the design's content is otherwise carried verbatim.

The governing spine is the post-RV-132 consequence model: a recursive
`needs`-leverage term over the acyclic dep backbone (depth-decayed, fan-out
summed, finite by the finite-DAG DP) plus a one-hop `ref`-optionality term over
the cyclic-capable lineage overlays, with `seq` left a structural constraint.
That model and its coefficient role/domain split are durable policy — hence they
are ratified (PHASE-01) before any code implements them.

## Sequencing & Rationale

**PHASE-01 — ADR-015 first (deviation from design §9 P5).** Design §scope says
ADR-015 is "authored during design phase and referenced by the implementation
plan" (i.e. early), while §9 lists it last. RV-132 F-4 resolved the tension in
the early direction: the coefficient *roles and domains* (`dep_coeff` a retention
factor in (0,1], `ref_coeff` flat non-negative, `seq` no weight class) are
ratified policy, not tunable defaults — they are *what makes D8/D9 valid*.
Ratify-before-implement is the correctness-safe order, and the ADR is independent
of all code, so it leads. Its content already exists, locked, in design §7; this
phase is the extraction/formalisation, not new decisions.

**PHASE-02 — risk leaf extraction.** The first code phase, and foundational:
`facet` (leaf) and `priority::graph` (engine) must both read risk data that today
lives in `backlog` (command). Reading upward violates ADR-001, so the risk types
move to a new leaf `src/risk.rs` (D2). This is the ADR-001 *forcing function* —
the binding tier-map (`layering.toml`, consumed by `just gate`) is amended in the
same phase (`risk = "leaf"`, `facet` comment relaxed), or the gate fails
(RV-121/SL-132 was caught on exactly this). Behaviour-preserving: `backlog`
re-uses the leaf and its existing risk/exposure suites are the proof (VT-1).

**PHASE-03 — read the two pure inputs.** Both scoring inputs come from disk and
both are independently testable, so they land together but *before* any consumer:
the scan reads the `[facet]` table into `ScannedEntity.risk` (preserving the
per-facet isolation contract, VT-1b), and `priority::config` parses `[priority]`
with the deliberate advisory-config clamp policy (silent clamp, never fatal —
distinct from `dispatch_config`; VT-3). Config is *not* threaded into `build_from`
this phase: with no scoring consumer yet, threading it would leave dead code or an
unused-binding warning. It is unit-tested standalone instead. `priority::config`
is classified `leaf` in the tier-map here, where the module is introduced.

**PHASE-04 — scoring engine, additive (deviation from design §9 P3 boundary).**
Design §9 folds "drop `PriorityGraph.consequence:u32`" and "replace the
consequence pre-pass" into P3, but the surfaces read that field until P4 — so a
literal P3 would not compile. This plan applies a strangler step instead:
PHASE-04 is *purely additive* — it adds `base_score`, `NodeAttr.base_score`, the
recursive-leverage DP (with the explicit dep-component condensation read from the
public `provenance().cycles()`), and the one-hop optionality pass, storing the
new `leverage`/`optionality`/`score` maps — while the **old** `consequence:u32`
field, pre-pass, mint order, and every surface stay untouched and green. The new
engine is exercised by direct graph/score inspection (VT-2/4/4b/6/8), so the
correctness-critical maths is proven in isolation, with the full existing suite
and goldens as an unchanged behaviour-preservation backstop, before any surface
churn. The threading of config into `build_from` (D4) lands here, where it has a
consumer.

**PHASE-05 — atomic cutover + activation.** Everything that changes observable
ordering happens in one phase, so the behaviour shift is single and deliberate:
retype the surfaces to `score: f64`, flip the mint tiebreaker to `base`, delete
the old `consequence:u32` path (leaving no parallel implementation), give `next`
its own score-aware frontier sort, add the score columns, bump `policy_version`
to v3, update the goldens, and seed an illustrative `[priority]` in
`doctrine.toml`. The `next` ordering is the one place to watch: it is a Kahn-style
induced-frontier sort over the *surviving* seq edges (`seq_overlay` minus
`provenance().evictions()`) with `(score desc, id)` as the ready-set priority —
**not** cordage's `order_key`, which ranks longest-path Level before NodeId and
would demote a score-promoted seq-successor below every low-score level-0 item
(RV-132 F-3, raiser-confirmed). VT-5 proves the reordering the slice exists to
produce; VT-7 proves the mint/display/`next` ordering split including the
evicted-seq guard.

## Notes

- **Deviations from design §9 are sequencing only, not scope.** PHASE-01-first
  (ADR ahead of impl) and the additive PHASE-04 / cutover PHASE-05 split both
  serve green-at-every-phase and ratify-before-implement; no design decision is
  altered. If governance prefers ADR-015 last, PHASE-01 can move with no
  dependency change (nothing depends on the ADR *file*).
- **Every phase ends green and gated.** `just gate` (workspace clippy + tests)
  passes at each phase boundary; the tier-map edits ride the phase that
  introduces the module they classify (PHASE-02 risk, PHASE-03 priority::config).
- **Behaviour-preservation backstop.** PHASE-02 (risk move) and PHASE-04
  (additive engine) both rely on the *existing* suites staying green unchanged as
  the proof of no regression; only PHASE-05 deliberately rewrites goldens.
- **Gate-green vs. scaffold-then-consume.** `just gate` runs lib-only clippy (no
  `--all-targets`, AGENTS.md) at zero warnings, so a field/struct introduced in
  phase N but consumed only in N+1 trips `dead_code` "never read" — its only
  readers are tests the gate doesn't compile. This is a per-phase artefact (the
  single-slice design never hit it; everything is consumed by slice-end). The
  mechanism, carried in each phase's EX criteria: house-style **`#[cfg_attr(not(test),
  expect(dead_code, reason = "consumed PHASE-NN"))]`** on the not-yet-consumed
  addition — `expect` not `allow`, because it is **self-clearing** (when the
  consumer lands the expect fires *unfulfilled*, forcing its own removal — debt
  cannot rot). PHASE-04's consumers clear PHASE-02/03's, PHASE-05's clear PHASE-04's,
  none survives the slice (PHASE-05/EX-5). Before reaching for the expect, the
  cheaper check: a field read by a `#[derive(Debug/Clone/PartialEq)]` impl is
  already "used" and will not warn — e.g. `EntityFacets` derives Debug+Clone, so
  its new `risk` field likely needs no suppression at all. This is the main
  execution-time wrinkle the small-phase split buys in exchange for reviewing the
  correctness-critical engine in isolation; it is deliberate, not drift.
- **Assumptions to confirm at execution** (carried into `/phase-plan`, not yet
  source-verified): (1) `provenance().cycles()` is populated for the `dep_overlay`
  specifically (it is the `Reject` overlay — design §3) and exposes member
  `.nodes()` as `EntityKey`-mappable NodeIds; (2) `provenance().evictions()`
  filters cleanly to the seq overlay for PHASE-05's surviving-edge set; (3)
  `graph.ordered()` yields the dep-overlay topological order the reverse-DP needs
  (vs the composed display order — domain_map invariant). These are the cordage
  seams the post-pass and `next` sort stand on; each is checked before its phase
  executes.
