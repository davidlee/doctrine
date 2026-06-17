# PRD-014: Estimation

<!-- Reference forms: entity ids padded (REQ-059, ADR-004); doc-local refs bare
     (OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## 1. Intent

Doctrine governs a graph of work — slices, requirements, specs, revisions, backlog
items, epistemic records — but carries no first-class way to say *how much human
attention an entity is likely to demand* before the work moves. That judgement lives
in chat history and reviewers' heads, so it cannot be queried, rolled up across a
subgraph, or compared against what the work actually cost.

An **estimate** answers this: a bounded, project-local claim about the **human
attention burden** required to move an entity through interpretation, design,
coordination, execution, verification, audit reconciliation, and close. It is *not*
a prediction of wall-clock time, labour hours, velocity, or a delivery commitment —
the coding phase may be a small part of it. Its value is decision support: it helps
answer whether an entity is too large or ambiguous to reason about, whether work is
safe to dispatch, whether more design or decomposition is needed first, and where
uncertainty and coordination burden concentrate across the graph.

The desired end state is a thin, optional facet that records two bounds, validates
when present, displays cleanly, and is exposed to graph tooling — defensibly
compatible with PERT-flavoured reasoning without importing project-management
ceremony. It is an input to scope control, dispatch safety, epistemic calibration,
audit planning, graph analysis, and simulation; it is never a gate, timesheet, sprint
primitive, or commitment.

## 2. Scope

In scope:

- An optional `[estimate]` facet carrying two bounded effort claims (`lower`,
  `upper`) in a project-wide unit, attachable to any TOML-backed addressable entity.
- Structural validation of a *present* estimate; absence is always valid.
- A project-wide estimation unit with a default, configured once, not per entity.
- Detailed-display rendering of the estimate, with optional derived spread (ratio and
  absolute width) in verbose/review views.
- A stable, policy-agnostic contract that exposes estimate metadata to graph tooling
  (Cordage) for downstream aggregation.
- A schema deliberately generic enough that later adoption by additional entity kinds
  is mechanical rather than a remodel.

Out of scope:

- **Aggregation policy.** How estimates roll up across a subgraph (additive vs
  path-sensitive vs explanatory vs replacement traversal, named policies such as
  rollup/critical-path/uncertainty-map/dispatch-readiness/audit-burden) is a
  downstream/Cordage concern, never encoded in the facet.
- **Simulation and prediction.** Monte Carlo sampling, distribution inference,
  week-by-week milestone forecasting, and velocity/actuals calibration are downstream
  analyses, not core. Analysis output never writes back to `[estimate]`.
- **Time and process machinery.** Wall-clock tracking, timesheets, sprint velocity,
  scrum commitment semantics, cycle-time/actuals capture.
- **Classification and gating.** Automatic wide-range thresholds, split-pressure
  classification, estimate gates for dispatch/execution/audit/close, and any
  "expected estimate" config. Normal display must not classify spread as wide, risky,
  or split-worthy.
- **Richer estimate fields in v1.** No `mode`, `distribution`, confidence, rationale,
  timestamp, or entity-local unit; no phase-by-phase execution estimates; no
  PERT-specific required fields. The full non-goal list is enumerated in §6 / the
  brief and binds downstream slices.

Boundary: the facet records *local bounded attention burden*. It does not decide how
the graph is rolled up, whether an estimate means "too large" or "unsafe", or when an
estimate has gone stale — those are caller/policy interpretations layered on top.
Estimate *drivers* (unknowns, risks, assumptions, open questions) are not duplicated
inside `[estimate]`; they are normal graph relations the facet stays thin against.

## 3. Principles

- **An estimate is attention burden, not duration.** It captures expected cognitive,
  coordination, verification, and reconciliation effort — not one person's
  uninterrupted labour, not a throughput target, not a commitment.
- **Optional everywhere; absence never blocks.** Doctrine may surface absence as a
  generic nudge, but it must never block work because bookkeeping about the work is
  missing. A missing estimate is not a schema violation.
- **Optional means omittable, not malformed.** A present `[estimate]` is either
  structurally valid or a hard validation error — there is no lenient repair.
- **The facet stays thin; epistemics explain the spread.** Estimate width is
  explained by linked unknowns/risks/assumptions/open questions through normal
  relations, never by fields copied into `[estimate]`.
- **Boring, stable, graph-shaped.** The facet records local bounds and exposes them;
  it does not interpret them. Aggregation, simulation, and prediction are someone
  else's decision, kept out of the facet so they cannot ossify into it.
- **Policy-agnostic exposure.** Tooling that consumes estimates (Cordage) gets
  structural primitives, never a doctrine-baked verdict on readiness, risk, or
  lateness.
- **Forward-compatible by construction.** The v1 two-bound shape must extend (e.g. a
  later `mode`/`distribution`) without invalidating any estimate authored under v1.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded as
requirement entities and appear under the synthesized Requirements section below.
This section carries only the constraints and invariants that bound every valid
implementation.

Constraints:

- The estimation unit is project-wide, configured once (`[estimation].unit`); there is
  no entity-local `estimate.unit` in v1. When config omits it, the unit defaults to
  `high_caffeine_hours`.
- `lower` and `upper` may be authored as TOML integers or floats; the internal model
  normalizes both to finite floating-point values.
- Estimate drivers must not be duplicated inside `[estimate]`; they are expressed as
  normal entity relations (e.g. `has_unknown`, `has_risk`, `has_assumption`). A
  dedicated `estimate_informed_by` relation is not added until ordinary epistemic
  relations prove insufficient.
- The facet must not encode aggregation policy, traversal semantics, simulation,
  thresholds, classification, or milestone prediction.
- Tooling that aggregates estimates (Cordage) must remain policy-agnostic: it exposes
  structural aggregation primitives and never decides "too large", "unsafe", "late",
  "blocked", or "ready".
- Simulation/analysis output is never persisted back to `[estimate]`; only an explicit
  human change may rewrite the bounds.

Invariants:

- A present `[estimate]` is either structurally valid or a hard validation error —
  `lower` and `upper` are both required, finite, with `lower >= 0` and
  `upper >= lower`; `NaN`/`±Infinity` are rejected.
- An absent `[estimate]` is always valid and never blocks any workflow.
- Displayed bounds are normalized compact values; original TOML numeric formatting is
  never preserved through display.
- A v1-valid estimate remains valid under any future facet extension (added optional
  fields never invalidate the two-bound form).

## 5. Success Measures

- From an entity alone, an agent or reviewer can read its expected attention burden
  and the *width* of its uncertainty, in the project's configured unit.
- A wide estimate is explainable by the entity's linked epistemic records (unknowns,
  risks, open questions) reachable through normal relations — not by anything inside
  `[estimate]`. A narrow estimate with no basis is recognisably more suspect than a
  wide estimate with clear epistemic links.
- No workflow — dispatch, execute, audit, close — ever blocks, warns fatally, or
  refuses because an estimate is absent or wide. Absence is surfaced, never enforced.
- Graph tooling (Cordage) can consume estimate metadata (id, kind, lower, upper, unit,
  relations, lifecycle state) and build aggregation primitives without understanding
  any doctrine-specific project-management semantics.
- Attaching the facet to a new entity kind is mechanical: it requires wiring the
  parse/hydrate path, not remodelling the estimate schema.
- A malformed present estimate is rejected loudly at validation; a structurally valid
  one round-trips unchanged through parse, hydrate, and catalog.

## 6. Behaviour

Primary flow — record an estimate: an author adds an optional `[estimate]` table with
`lower` and `upper` to an entity's TOML. Integers or floats are accepted and
normalized to floats internally. The entity remains valid with or without the table.

```toml
[estimate]
lower = 2.0
upper = 8.0
```

Validation flow: when `[estimate]` is present it must be structurally valid —
`lower` required, `upper` required, both finite TOML integers or floats,
`lower >= 0`, `upper >= lower`. Invalid cases (missing `lower`/`upper`, `NaN`,
`±Infinity`, negative `lower`, `upper < lower`) are hard validation errors. When
`[estimate]` is absent, validation passes — absence is not malformation.

Configuration flow: the unit is read from project config `[estimation].unit`; when
omitted it defaults to `high_caffeine_hours`. The unit is project-wide; entities carry
bounds, never their own unit.

Detailed-display flow: detailed entity display shows the estimate when present and a
fixed phrase when absent.

```text
Estimate: 2-8 high_caffeine_hours
Estimate: none recorded
```

Verbose/review display may add derived spread — attention ratio and absolute width.
When `lower == 0` the ratio is unavailable (division by zero), reported as such; width
is still shown.

```text
Estimate: 2-8 high_caffeine_hours      Estimate: 0-4 high_caffeine_hours
Attention spread: 4x                   Attention spread: ratio unavailable
Attention width: 6 high_caffeine_hours Attention width: 4 high_caffeine_hours
```

Normal display must not classify spread as wide, very wide, risky, or split-worthy.
Review-oriented or explicitly configured views may do so; the default must not.

List/status flow: list and status views stay sparse. A default compact row may carry a
single generic attention marker (`!`) only when a caller or local convention decides a
nudge is useful; the marker is generic (it may mean "no estimate recorded" or "linked
knowledge records bear on active work"). Verbose list output may explain the marker.
The marker is a caller/local-convention concern, not a gate, and v1 adds no config for
whether an estimate is expected.

Graph-exposure flow: catalog/graph hydration exposes each estimated node's metadata —
entity id, entity kind, estimate `lower`, estimate `upper`, project estimation unit,
relations/graph edges, and lifecycle state where available — so Cordage can provide
traversal and aggregation primitives without doctrine baking in rollup policy.

Edge cases and failure modes: `lower == 0` yields an unavailable ratio but a valid
estimate; equal bounds (`lower == upper`, zero width) are valid; a present-but-malformed
table fails validation rather than being silently repaired; an unestimated node inside
an otherwise-estimated subgraph is a display/overlay concern for callers, never an
error.

## 7. Verification

Verification confirms that the facet records and validates bounded estimates, displays
them faithfully, never gates work, and exposes a stable contract to graph tooling —
without binding the spec to a particular implementation.

Validation is proven by exercising the accept/reject matrix directly: a present
estimate with finite `lower >= 0` and `upper >= lower` (as integers and as floats)
validates and normalizes to floats; each malformed case (missing bound, `NaN`,
`±Infinity`, negative `lower`, `upper < lower`) is rejected as a hard error; an absent
estimate validates. Round-trip durability is proven by confirming a valid estimate
survives parse, hydrate, and catalog projection unchanged. The default-unit behaviour
is proven by reading the unit with and without `[estimation].unit` configured.

Display is proven by rendering: present (`Estimate: 2-8 <unit>`), absent
(`Estimate: none recorded`), verbose spread (ratio + width), and the `lower == 0`
ratio-unavailable case — and by confirming normal display never emits a wide/risky/
split classification. The non-blocking guarantee is proven structurally: no
dispatch/execute/audit/close predicate reads estimate presence — absence of such a
read is the proof, not a passing run. The graph contract is proven by confirming
catalog/graph hydration exposes the required per-node fields for a Cordage consumer.

Where a check must reference a specific obligation, it cites the durable requirement
entity (REQ-NNN), never a mobile `FR-`/`NF-` membership label. Coverage of the
functional and quality requirements is tracked against those entities, not duplicated
here.

## 8. Open Questions

<!-- Number bare: OQ-1, OQ-2, … (glossary § reference forms). -->

- OQ-1 — The generic `!` attention marker in list/status views: should v1 ship any
  doctrine-surfaced nudge at all, or leave the marker entirely to caller/local
  convention? Blocks list-view behaviour and its acceptance tests. Current posture:
  caller/local-convention, no v1 config.
- OQ-2 — Which entity kinds expose estimate *input* in v1? The schema is kind-agnostic
  and slice is the recommended first consumer; the remaining likely consumers
  (requirement, product/tech spec, revision, backlog item, unknown, assumption, risk,
  open question, test plan, audit/review packet) adopt it mechanically later. Which,
  if any, beyond slice are in the first slice's scope?
- OQ-3 — Should the Cordage-facing aggregation contract (traversal policies, named
  rollup policies, path-sensitivity) be captured as a technical specification (SPEC)
  downstream of this PRD, separate from the facet itself? It is out of scope here but
  needs a home before aggregation is built.
- OQ-4 — Staleness has no v1 mechanism (no timestamp/confidence fields). When material
  facts change (unknown resolved, risk retired, scope/design rewrite, audit rework),
  tools should treat estimates as potentially stale. If lifecycle/provenance signalling
  is later wanted, it should reuse a generic mechanism rather than expand `[estimate]`
  into a planning record — which mechanism?
- OQ-5 — `estimate_informed_by` is deliberately deferred: do ordinary epistemic
  relations (`has_unknown`/`has_risk`/`has_assumption`) suffice for tooling to infer
  estimate drivers, or will a dedicated relation eventually be needed? Resolve only if
  the generic relations prove insufficient.
