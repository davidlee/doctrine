# Elicitation queue

## Context

RFC-019 Phase C. Phases A (SL-210, capture) and B (SL-213, inference) are
shipped; the entry criterion — empirical evaluation of Phase B against a real
ledger over this repo's backlog — is met with caveats (CHR-042,
`.doctrine/rfc/019/phase-b-evaluation.md`). The eval's caveats C1–C3 are
direct design inputs here, not background. Originates from IMP-280.

Nothing yet *asks* for judgements: evidence accrues only when someone
volunteers a `doctrine compare record`. This slice builds the selection side —
a deterministic candidate queue over the Phase B constraint layer, plus the
capture loop that renders a candidate into an answerable question and appends
the answer to the ledger.

Five scope-shaping questions were dispositioned at preflight (2026-07-12) and
are baked in below: Q1 naive queue-head ships first; Q2 the ConstraintSet
query API is this slice's design work; Q3 stale-anchor reprobe is first-class;
Q4 the engine stays value-only; Q5 human-confirmation preference without the
D7 demotion knob.

## Scope & Objectives

Strictly additive; queue pure over `(ConstraintSet, costs, ledger, statuses,
context)`; disk stays at the existing scan seam.

- **Query API over `ConstraintSet`** (SL-213 D12 anticipated this: "Phase C
  designs its own predicates"). Three predicates, in-crate beside
  compile/project:
  - `determined(a, b)` — is the pair ordered under *every* feasible
    assignment? Joint-set reasoning: reachability over the condensed DAG +
    anchor-implied order. Never computed from the interval box (SL-213's
    D-bounds warning stands).
  - hypothetical-apply → yield count — apply a candidate answer, recompile
    the augmented active set (reuse `compile`; it is pure and evidence-sized
    — no second propagation engine), count newly-determined decision-relevant
    pairs. Guaranteed yield = minimum over the candidate's answer space.
  - indeterminate-pair enumeration within the candidate pool (decision-
    relevant region only — O(K²) for planning depth K, not corpus-quadratic).
- **Candidate queue** (RFC-019 §Pair selection): ranked by guaranteed-yield ×
  decision-impact, reasons attached to every candidate, deterministic within
  its bounded candidate policy, JSON-emittable for an agent curator.
  - **Sequencing decision context** ships (the internal default: frontier
    top-K order-stability). Scoping context (budget cut) is Phase E.
  - **Stall semantics per the 2026-07-11 review**: zero one-step guaranteed
    yield means the greedy heuristic is exhausted, NOT that the decision set
    is stable; the render says which. Stability claims only via the
    determinacy predicate over the joint set.
  - Un-compared items enter by binary insertion against the projected median.
  - **Q1**: the queue head is the naive selector and ships first; curation
    (commensurability-in-the-small, audience fit, frame choice, session flow)
    is agent-skill territory, not engine code.
- **Stale-anchor reprobe as first-class queue candidates (Q3 / eval C2).**
  Over the dense fiat-anchor corpus, one stale anchor sterilised 28% of the
  ledger via D4 closure — the evidence budget must not pool in quarantine.
  Anchor-conflict findings become engine-ranked queue entries with computable
  hypothetical answer space ({anchor edited → quarantined rows activate} vs
  {anchor upheld → rows superseded/tombstoned}), yield-ranked by the same
  machinery as comparisons. Value-domain only — no Phase E dependency. This
  is the one deliberate scope addition over the RFC's Phase C text; eval C2
  is its warrant.
- **Human-confirmation selection preference (Q5, T7).** Impact weighting
  prefers candidates where a human would confirm a load-bearing agent-seeded
  ordering (`RaterCounts` already discloses `0h/Na`). The D7 rank-aware
  quarantine/demotion knob stays out (later seam, pre-Phase-D obligation).
- **Capture loop.** Elicit verb surfaces the ranked queue; renders both items
  with enough context to choose (title, body summary, deps, risk); accepts
  the answer; appends through the existing Phase A capture path
  (`compare record` mechanics). Estimate/value edits re-surface reprobe
  candidates through the same determinacy check.
- **Bare-estimate annotation (Q4 / eval C1).** Projection's payoff in
  `next`/`survey` is estimate-gated (`value_dim = value/est_cost`; bare items
  anchor to `max_upper + margin` and sink regardless of projected value).
  The queue/explain surface annotates the mask ("projection masked by bare
  estimate"); nominating the estimate itself is curator-level heuristic, not
  engine yield-ranking.
- **Verb naming** — design decision. RFC sketches `doctrine value elicit`;
  Phase A settled a top-level `doctrine compare` group (SL-210 D1). Settle at
  design against STD-002 and that precedent.

## Non-Goals

- Scoping decision context (budget water-line, cut report, membership
  stability) — Phase E, gated on the estimate feasible-region model
  (RV-260 F-5).
- Engine yield-ranking of estimate questions (Q4) — same Phase E gate;
  curator nomination only.
- D7 rank-aware quarantine / agent-testimony demotion knob (Q5) — later seam,
  mandatory before stakeholder surfaces (Phase D+), not built here.
- Session mode, stakeholder web surface, tension narrative — Phase D.
- Agent curation itself (the skill layer) — ships as skill text when the
  queue exists; engine ships the JSON surface it consumes.
- Ratio-frame elicitation (OQ-6) and `priority`-domain frames — out; value
  domain `equal-effort` only, matching the shipped constraint compiler.
- Any governance edit.

## Affected surface

- `src/comparison/` — new module(s) for the query predicates + queue
  (exact home at design; ADR-001 layering — pure core, no disk/clock).
  `compile.rs` may gain narrow read accessors; no semantic change to
  compile/project (behaviour-preservation gate).
- `src/commands/` — new elicit verb (name at design); `compare.rs` capture
  path reused for the answer leg.
- `src/priority/findings.rs` — anchor-conflict findings feed reprobe
  candidates; possible new annotation variant for the bare-estimate mask.
- `src/priority/surface.rs` / `render.rs` — queue render, stall/stability
  wording, bare-estimate annotation.
- Tests: determinism suite for the queue (same merged file set ⇒ same ranked
  queue); yield-computation unit battery; reprobe-candidate propagation;
  binary-insertion entry; stall-vs-stable render.

## Risks, assumptions, open questions

- **Risk (medium, the RFC's own "algorithmic risk"):** question quality —
  greedy one-step yield may surface formally-informative but humanly-junk
  pairs. Mitigation: reasons-attached rendering, curator layer above, Q1
  ship-first posture keeps the engine surface small.
- **Risk (low):** recompile-per-hypothetical cost. RFC bound:
  O(K² · |active rows|) per refresh, trivial at realistic sizes. Assumption
  carried from preflight; measure at VT if a corpus proves otherwise.
- **Assumption:** anchor-review and comparison candidates ride one ranked
  queue surface, kind-tagged (design confirms the JSON shape).
- **Assumption (eval C3):** the cycle path's standing assurance is the
  SL-213 prototype battery; a deliberately elicited contested triad becomes a
  dogfood follow-up once human raters are in the loop — not a blocker here.
- **OQ:** frontier depth K ownership — config vs per-invocation flag
  (design).
- **OQ:** queue-entry JSON schema for mixed candidate kinds (design).

## Verification / closure intent

- **Behaviour preservation:** no ledger / no invocation ⇒ every existing
  priority and comparison suite passes unchanged; compile/project semantics
  untouched.
- **Determinism:** same merged file set + statuses + config ⇒ identical
  ranked queue, byte-stable JSON; no clock/rng in the pure layer.
- **Determinacy predicate:** joint-set correctness — pairs ordered by
  chain+anchor evidence report determined; interval-box-overlapping but
  chain-ordered pairs report determined (the box is not the oracle);
  genuinely open pairs report indeterminate.
- **Yield:** hand-computed small-graph battery — guaranteed yield equals the
  min-over-answers count of newly-determined decision-relevant pairs; zero-
  yield bridge case renders stall (not stability).
- **Reprobe:** a synthetic stale-anchor conflict yields an anchor-review
  candidate ranked by its un-quarantine payoff; resolving it (edit anchor)
  activates the closure rows on recompile.
- **Capture loop:** elicit → answer → row appended via Phase A mechanics →
  next refresh consumes it (end-to-end).
- **Human-confirmation preference:** agent-only load-bearing ordering ranks
  above an equivalent human-confirmed one, all else equal.
- **Bare-estimate annotation:** projected-but-bare item renders the mask
  line; estimated item does not.

## Summary

Makes Phase B's inference actionable: a deterministic guaranteed-yield ×
decision-impact candidate queue over the retained ConstraintSet (its first
query API), with stale-anchor reprobe promoted to first-class candidacy on
the eval's C2 evidence, a capture loop closing the ask→answer→ledger cycle,
honest stall-vs-stable semantics, and the estimate-side payoff gap surfaced
rather than silently absorbed — engine queue first, agent curation layered
above, per the RFC's queue/curator split.

## Follow-Ups

- Curation skill (commensurability, audience fit, session flow) over the
  JSON queue surface.
- Dogfood: deliberately elicited contested triad to exercise the D3 cycle
  path empirically (eval C3) once human raters participate.
- Phase D: tension narrative + session surfaces; D7 demotion knob before
  stakeholder exposure.
- Phase E: estimate feasible-region model → scoping context + cross-domain
  yield ranking; estimate-domain sibling batch precedes it.
