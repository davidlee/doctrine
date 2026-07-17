# SPEC-024: Comparison engine

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## 1. Intent

The comparison engine is the elicitation and inference substrate behind
doctrine's `value` (and, for sizing, `estimate`) magnitudes: instead of a
single hand-typed float standing as truth, every judgement — relative
("A over B") or absolute ("A is worth 7") — lands as a dated, attributed,
supersedable row in an append-only ledger, and the priority engine's
`value_dim` consumes a derived scalar with disclosed provenance rather
than an opaque number. It realises RFC-019's evidence architecture
(Phases A–C: capture, inference, elicitation) and RFC-020's claim-
authority ladder (Phase 1: value anchors as ledgered evidence, tiered
above/below the comparison projection rather than binary anchors-win).
PRD-014 owns the *what*/*why* of value and estimate as facets; this spec
is the *how* of the evidence layer that now produces the value the facet
used to hold outright.

## 2. Scope

In scope: the session-file wire schema and its closed vocabularies
(domain, frame, form, rater, response); the three-tier pure pipeline
(resolve → compile → project) that turns ledger rows into a per-entity
value scalar with provenance; the claim-authority ladder that resolves
competing absolute magnitudes (pin/human/agent/migrated) into the
constraint layer's anchor map; the deterministic-degradation machinery
(preference-cycle and anchor-conflict quarantine) and its findings; the
elicitation candidate queue (`compare elicit`); the `doctrine compare`
verb group and the `value set|pin|clear` capture path that rides the
same wire model; and the seam into `priority::graph` that consumes the
resulting projection and cost feed.

Out of scope, ridden by name below (§8) because RFC-020's transition is
open: estimate-domain **anchor** claims (RFC-020 Phase 2 — `[estimate]`
facet retirement); REQ/PRD/spec-container value admissibility (RFC-020
Phase 3, hierarchy); cross-level aggregation/coherence diagnostics;
ratio-form judgement compilation (RFC-019 OQ-6); the stakeholder-facing
web elicitation surface (RFC-019 T4, Phase D+). The estimate **domain's
pairwise ordering** (SL-219, `more-work` frame, `est_cost` feed) IS
shipped and in scope; only its anchor-claim form is not.

## 3. Principles

- **Elicit base, derive consequence (RFC-019 A1).** A judgement targets
  an item's intrinsic worth only — never what it unlocks or what depends
  on it; leverage and burndown stay downstream, derived once, in
  `priority::graph`.
- **Evidence is lossless; interpretation degrades deterministically.**
  Tier 1 (the ledger) never discards, merges, or reinterprets a row.
  Tier 2 (compile) may quarantine a row under a contradiction, but
  always by a named, deterministic rule, always surfaced as a finding —
  never by silent smoothing and never by going infeasible.
- **Authority is derived, not authored.** No row carries a free
  authority field; a claim's tier follows from its mandatory `rater`
  plus, for `pin`, an explicit operator-gated admission path — never
  from the writer's self-assertion (RV-275 F-5).
- **Correction is supersession.** Nothing is edited in place; a row is
  superseded by naming it (`supersedes`) or withdrawn by an appended
  tombstone. The ledger's version history is its append log.
- **Closed vocabularies, not free text.** Domain, frame, form, rater,
  and response are closed enums; an unknown frame/domain token still
  parses and round-trips losslessly (forward-compatible), but the
  constraint compiler only ever consumes the vocabulary it understands.
- **Pure core, one impure seam.** Every tier below `store` is a pure
  leaf module (ADR-001) — no clock, disk, rng, or git; `store.rs` is the
  sole disk-touching seam, composing `load_sessions` → resolve → compile
  → project behind one call.
- **No score is authored to disk.** Projected values, bounds, and
  determinacy are always recomputed from `(ledger, config, anchors)` —
  never snapshotted, mirroring ADR-015's scoring posture at this layer.

## 4. Requirements

Constraints and invariants the engine holds — the functional and quality
requirements themselves are separate `REQ-NNN` entities (this spec's
membership), not prose rows here.

Every judgement row is immutable once written to a session file; the
only legal corrections are an explicitly superseding row (same identity
key: pair or subject, domain, frame, form, lens, rater) or an appended
tombstone naming the withdrawn row's uid — no in-place edit path exists
anywhere in the pipeline. A same-tier conflict (two active claims at the
same authority tier with incompatible magnitudes) never resolves
silently: it is a `ClaimFinding::Conflict`, never a lexicographic or
latest-wins pick, and a Pin/Human conflict additionally nominates a
reprobe. Quarantine (preference-cycle or anchor-conflict) is symmetric —
every row on the violating structure demotes, none silently drop — and
the system it produces is always satisfiable; a contradiction is
surfaced, never allowed to make the constraint set infeasible. Every
ordering-sensitive computation in the pure tiers keys off
`BTreeMap`/`BTreeSet` with a uid or canonical-id tiebreak; no tier
compares or keys by raw `f64` (`total_cmp` only where a float order is
needed), so output is independent of session-file read order and of
which developer's machine wrote which file first. An entity's magnitude
is never asserted unconditionally by capture: a bare `f64` guards
`NaN`/`±Infinity` at the wire boundary, and every anchor claim's
mandatory `rater` closes off the "who said this" gap the pre-ledger
`[value]` facet left open.

## 5. Success Measures

- Every existing priority-engine suite passes unchanged with zero
  comparison sessions on disk (`load_pipeline` short-circuits to an
  empty pipeline) — the behaviour-preservation gate RFC-019/RFC-020
  both name.
- A determinism suite (mirrored from the no-NaN/total-order priority
  tests) proves every pipeline output is stable under row-set
  permutation and under which of two developers' session files a merge
  sees first.
- The SL-220 migration census accounts for every legacy `[value]` facet
  as a `migrated`-tier claim: zero silent provenance conversions,
  lossless rollback, every facet represented (the shipped run: 185
  migrated, zero discrepancies).
- The projection port's golden scenario battery (S1–S8, Y1–Y7, N1–N4,
  ported from the design prototype) stays byte-identical across changes
  to the pipeline's non-gauge-affecting internals.

## 6. Behaviour

### The wire model and session ledger

A comparison **session file** (`.doctrine/comparisons/<date>-<uid>.toml`,
schema `doctrine.comparison-session`, current version 3) carries a
`[session]` header (uid, date, optional audience) plus append-only
`[[judgement]]` and `[[tombstone]]` arrays. A judgement row (`Judgement`,
`src/comparison/wire.rs`) is either **pairwise** (`form = order|ratio`:
two subjects `a`/`b`, a closed `response` — `prefer-a|prefer-b|equal|
incomparable`) or a single-subject **anchor** (`form = anchor`: an
absolute `magnitude` claim, no `b`/`response`) — the two shapes share
one row type, one file, one supersession/tombstone machinery. Every row
carries a mandatory `rater` (`human|agent|migrated`), an optional `lens`
(the multi-dimensional-value seam, inert until IDE-035), and either
`date` (live capture) or `observed_at` (migrated rows only — the honest
absence of an authorship date). `domain` and `frame` are `String`-typed
for forward-compatible round-tripping, but `frame` derives `domain`
through one closed table (`DOMAIN_FRAMES`) at capture — a user never
types a domain. Three domains ship: `value` (`equal-effort` /
`value-anchor` frames), `priority` (`prefer-first` — capacity-cutoff
testimony, deliberately never compiled to a value constraint), and
`estimate` (`more-work` — settle-cost ordering, winner is the costlier
side).

### Session flow and capture

`doctrine compare record <A> <B> --prefer|--equal|--incomparable
[flags]` (`src/commands/compare.rs`) is the impure shell: it resolves
both refs to canonical ids via the facet resolver (rejecting a dangling
or bare-id ref), derives the domain from `--frame`, validates
`--supersedes` against the loaded corpus (an unknown target uid is the
one hard capture-time error), mints the impure edge (uuid v7 uids,
today's date), and writes a fresh session-of-one file — one capture, one
file, clobber-refusing, so two developers' concurrent sessions merge as
clean add/adds rather than racing an EOF. `compare list` / `compare
withdraw` read and evict by uid; `compare elicit` is read-only end to
end (no TTY interaction, no writes). `doctrine value set|pin|clear`
(`src/commands/facet.rs`) mints anchor-form rows through the *same*
record path: `value set` requires `--rater` explicitly (no default — a
default would fabricate provenance); `value pin` requires `--by` and is
refused outside an interactive operator session (worker-mode guard) —
the pin tier is a deliberate, attributed, gated act, never a side effect
of typing a number.

### The evidence model: resolve → compile → project

The pure pipeline (`src/comparison/{resolve,compile,project}.rs`, wired
by `store::load_pipeline`) is three tiers:

1. **Resolve** (tier 1, row validity) tags every row with exactly one
   `ResolutionStatus` — `Active`, `Superseded { by }`, `Tombstoned`,
   `InertLens`, `InertDomain`, `InertLifecycle`, or `Malformed` (a
   supersession-cycle participant) — over the parsed sessions plus an
   entity-status map, in one order-free pass. Supersession is a durable
   act: `supersedes` wins outright; within a single session file the
   same rater re-answering later (higher `seq`) is an implicit
   revision; the same identity key arriving in *different* session
   files is concurrent, both-active evidence, not a lexicographic pick.
2. **Compile** (tier 2, constraint compilation) takes the active
   pairwise rows plus an `AnchorMap` and produces a `ConstraintSet`:
   equality-merged classes (an `equal` response merges two entities
   into one class), a strict winner→loser digraph, per-class anchors
   and display bounds, and a quarantine ledger. A **preference cycle**
   collapses its members into one tied class and quarantines every
   participating row (`QuarantineReason::PreferenceCycle`); a row whose
   edge contradicts a pair of differently-valued anchors quarantines as
   `QuarantineReason::AnchorConflict`. Both are named, deterministic,
   symmetric — never a silent drop, never an infeasible system.
3. **Project** (tier 3, placement) turns a `ConstraintSet` into a
   `Projection`: a scalar per evidence-bearing entity plus its
   `ValueProvenance` (`Authored`/`Projected`/`Gauge`). Placement is
   reverse-topological greedy, per weakly-connected component: an
   anchored class is exact; an unanchored class between a floor and a
   ceiling gets budgeted interpolation; an unbounded tail steps by a
   gauge step; a component with no anchor at all spreads around a gauge
   center (`DEFAULT_VALUE` for the value domain).

`query.rs` layers three predicates over a compiled `ConstraintSet` for
`compare elicit`: `determined` (is a pair's order fixed over the joint
feasible set — a closed-form check licensed by the shipped vocabulary's
marginal-exactness lemma, void once ratio/band rows land),
`hypothetical_outcome`/`hypothetical_yield` (apply a candidate answer,
recompile, report the signed determinacy delta), and
`indeterminate_pairs` (the bounded candidate pool for a queue refresh).

### Claim resolution: the authority ladder

`claims.rs` is a fourth pure fold, sitting between resolve and compile,
that resolves *anchor*-form value rows into `ClaimTier::{Migrated,
Agent, Human, Pin}` (ascending order, `Pin` outranks all).
`is_anchored()` is true only for `Pin`/`Human`: those route into
`ClaimResolution::anchored` — the exact `AnchorMap` compile consumes —
while `Agent`/`Migrated` route into `priors`, which bypass the
constraint layer entirely (the D4 anti-laundering split — an agent's
guess can shape a prior but can never anchor the constraint graph). A
same-tier disagreement (two active claims, incompatible magnitudes)
resolves to the tier's mean as the point value but is always flagged as
a `ClaimFinding::Conflict`; a Pin/Human conflict additionally nominates
a human reprobe. Lens-tagged anchor rows resolve identically but stay
inert to `value_dim` until IDE-035 (the RFC-019 T5 pooling discipline,
still in force).

### The value derivation and the seam into the priority graph

`priority::graph::effective_raw_value` is the single definition of an
entity's value for scoring: an anchored claim (Pin/Human tier) wins
outright; else the comparison projection (Projected or its Gauge
sub-tier); else an agent-tier prior, then a migrated-tier prior; else
the legacy `[value]` facet (transitional, consulted only when zero
unlensed claim rows exist); else `DEFAULT_VALUE` for a value-bearing
kind. This is the RFC-020 T3 ladder realised in code — the flip that
matters: agent-authored magnitudes sit as priors *below* the comparison
projection rather than as anchors above it. `priority::graph::build_from`
calls `comparison::load_pipeline` exactly once per build (one ledger
read feeds both the value projection and, via `comparison::cost_feed`,
the estimate-domain cost feed `est_cost` consumes as its second-tier
source) and threads the resulting `Pipeline` into `base_score`'s
`value_dim` computation and the elicit shell's `item_costing`.
Value-bearing kinds are `SL` plus the backlog kinds including `RSK`
(`src/kinds.rs`) — RSK's dual participation in both `value_dim` and
`risk_dim` is a known, adjudicated tension (RFC-020), not an oversight.

## 7. Verification

The pure tiers carry unit suites at each module boundary: `resolve`'s
determinism and supersession-cycle tests, `compile`'s rule battery
(equality merge, cycle collapse, anchor-conflict quarantine, symmetric
policy), `project`'s golden scenario battery ported byte-for-byte from
the design prototype (`.doctrine/slice/213/projection-prototype.py`),
and `claims`' permutation/duplicate-merge/cross-session/conflicting-pin/
lens-isolation battery (the RV-275 gate obligation). `query`'s
`determined` predicate is checked against a test-only backtracking
extension oracle. The `store` seam is proven behaviour-preserving: the
full existing priority suite runs unchanged against zero comparison
sessions on disk. The SL-220 migration carried its own census-verified
proof (185/185/0/0) rather than a spec-level coverage table —
verification here is evidence-in-code (tests, goldens, the census
script), not a status derived from this document.

## 8. Open Questions

RFC-020 (ledgered facet claims) is **open**; Phase 1 (value anchor
claims — this spec's claim-ladder mechanism) has shipped, but the
transition is not complete:

- **Estimate-domain anchor claims (RFC-020 Phase 2).** The `estimate`
  domain's *pairwise* ordering ships (SL-219); its **anchor** form does
  not — `anchor_frame_for` returns `None` for `estimate` today, and the
  `[estimate]` facet has not retired. Until Phase 2 lands, estimate
  magnitudes are still authored facets, not ledgered claims, and this
  spec's authority ladder does not apply to them.
- **Hierarchy value admissibility (RFC-020 Phase 3, OQ-3).** REQ/PRD/
  spec-container participation in the value domain, the pedigree-posture
  config (`off|advisory|strict`), and cross-level capture gating are
  designed in the RFC but not implemented; no code path admits a
  container as a comparison subject today.
- **Cross-level aggregation (RFC-020 OQ-1).** Package- vs
  portfolio-valued container aggregation modes, and any burndown/
  progress view built on them, are explicitly deferred behind an
  ADR-018 revision that has not been written.
- **Ladder × lens composition (RFC-020 OQ-2).** The full interaction
  between the authority ladder and lens-partitioned rows, beyond the
  fixed unlensed-pooling posture, is Phase-1-gate material the RFC
  marks unresolved.
- **Abstention as claim evidence (RFC-020 OQ-4).** Whether an
  `incomparable`-style "cannot value/estimate now" needs an
  anchor-claim analogue is unresolved.
- **Ratio-form judgements (RFC-019 OQ-6).** The wire schema carries
  `form = ratio` and a magnitude column, but no phase yet admits ratio
  rows at capture or compiles them — pure `order` semantics ship.
- **The stakeholder web elicitation surface (RFC-019 T4/OQ-1, Phase D+).**
  Sequenced as an intended direction, not yet designed.
