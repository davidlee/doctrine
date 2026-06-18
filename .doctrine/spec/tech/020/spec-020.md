# SPEC-020: Estimation facet

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The estimation facet is the mechanism realising **PRD-014**: an optional, bounded
`[estimate]` table that records the expected human attention burden an entity
implies. It is a component of the entity engine (**SPEC-004**) — a typed facet bound
onto the shared kind-agnostic materialiser, not a kind of its own. All shared
mechanism (the `<entity>-<id>.{toml,md}` identity pair, typed facet tables,
edit-preserving writes, catalog hydration) lives in the parent container and is used
here unchanged; this spec carries only what is specific to the estimate facet:
its model, normalization, validation, unit resolution, display, and graph exposure.
It also covers the **Value facet**: a single `f64` magnitude parsed from an optional
`[value]` table, with project-wide unit resolution from `[value].unit` defaulting to
`magic_beans`.

PRD-014 owns the *what* and *why* (the meaning of attention burden, the optionality
contract, the full non-goal list). This spec is the *how* and does not restate that
intent. Aggregation, simulation, thresholds, classification, and milestone
prediction are PRD-014 non-goals and are absent here by construction — the facet
records local bounds and exposes them; callers decide rollup.

## Responsibilities

Mirrors the structured `responsibilities` list: the `EstimateFacet` model and parse
seam, bound-normalization and the validation matrix, project-wide unit resolution,
round-trip durability, pure display rendering, the policy-free graph-exposure
contract, and the structural non-blocking guarantee.

### The `EstimateFacet` model and parse seam

A single reusable `EstimateFacet` carries two finite `f64` bounds, `lower` and
`upper`. It is parsed from an optional `[estimate]` TOML table off the entity's
identity file via the entity engine's facet seam — the same typed-facet-table
mechanism SPEC-004 already provides for other facets — so the model is defined once
and attached to a kind by wiring that kind's parse/hydrate path, never by
specialising the schema. The facet assumes no slice-specific shape. The model, its
normalization, its validation, and its rendering are **pure** (the architecture
rule: no clock, rng, git, or disk in the facet layer); only the project-config read
and hydration sit in the imperative shell.

### Normalization and the validation matrix

Authored bounds may be TOML integers or floats; both normalize to finite `f64` at
the parse boundary. A *present* `[estimate]` is then either structurally valid or a
hard parse/validate error — there is no lenient repair:

```text
lower required · upper required · both finite · lower >= 0 · upper >= lower
reject: missing lower · missing upper · NaN · +Infinity · -Infinity
        · negative lower · upper < lower
```

An *absent* `[estimate]` parses clean — absence is not malformation. Non-finite
values are rejected at normalization rather than admitted and flagged downstream.

### Project-wide unit resolution

The estimation unit is read from project config `doctrine.toml [estimation].unit`
(the same config surface as `[conduct]`), defaulting to `espresso_shots` when
unconfigured. The unit is project-wide; the facet schema carries no entity-local
unit field in v1.

### Round-trip durability

A valid estimate survives parse → hydrate → catalog projection unchanged: the
normalized `f64` bounds are stable across the pipeline. Original TOML numeric
formatting (e.g. `2` vs `2.0`) is not retained — the normalized form is the truth
that round-trips.

### Display rendering

Rendering is a pure function over the normalized model and the resolved unit:

```text
present:   Estimate: 2-8 espresso_shots
absent:    Estimate: none recorded
verbose:   Estimate: 2-8 espresso_shots
           Attention spread: 4x          (upper / lower)
           Attention width: 6 espresso_shots   (upper - lower)
lower==0:  Attention spread: ratio unavailable      (width still shown)
```

Bounds render as normalized compact values. Normal display classifies nothing — it
never labels a spread wide, very wide, risky, or split-worthy; a review-oriented or
explicitly configured view may, but the default must not. The list/status attention
marker (`!`) is a caller/local-convention concern, not part of this mechanism (the
v1 posture, PRD-014 OQ-1).

### The graph-exposure contract

Catalog/graph hydration exposes each estimated node's metadata through a stable,
policy-free contract: entity id, entity kind, estimate `lower`, estimate `upper`,
the project estimation unit, the node's relations/edges, and lifecycle state where
available. The contract carries no aggregation, traversal, or interpretation
policy — a consumer (Cordage) builds traversal/aggregation primitives over it
without understanding any doctrine-specific project-management semantics. Estimate
*drivers* are not duplicated into the facet; they are the node's ordinary epistemic
relations (`has_unknown`/`has_risk`/`has_assumption`), already on the exposed edges.

Vocabulary on the Cordage-facing surface must stay graph-neutral: the crate's
whole-word denylist (SPEC-001 Appendix B) forbids standalone product nouns — no
`project`, `task`, `schedule`, `capacity`. Use graph-neutral terms (`node`,
`member`, `value`, `width`, `overlay`).

### The structural non-blocking guarantee

No dispatch, execute, audit, or close predicate reads estimate presence. The
guarantee is **structural** — the absence of any such read — and is proven by that
absence, not by a passing workflow run. A missing estimate is never a validation
error and can never block, fatally warn, or refuse any workflow.

### The `ValueFacet` model and parse seam

A single reusable `ValueFacet` carries one finite `f64` magnitude, `value`. It is
parsed from an optional `[value]` TOML table via the same facet seam as the Estimate
facet. The model, its normalization, and its rendering are **pure**; only the
project-config read and hydration sit in the imperative shell. Integer and float
authoring both normalize to finite `f64`; non-finite values (`NaN`, `±Infinity`)
are rejected at the parse boundary.

### Value validation

A *present* `[value]` requires the `value` field present and finite — missing or
non-finite produces a hard parse error. There is no range validation (the Value
facet carries a single magnitude with no ordering constraint). An *absent*
`[value]` parses clean.

### Value unit resolution

The value unit is read from `doctrine.toml [value].unit`, defaulting to
`magic_beans` when unconfigured. The unit is project-wide; the facet schema carries
no entity-local unit field in v1.

## Concerns

- **Failure mode — silent repair.** The one hazard is admitting a malformed present
  estimate by coercing it (clamping a negative `lower`, swapping inverted bounds).
  The facet must fail loud at validation instead; repair would let bad bounds
  round-trip as truth.
- **Purity.** Normalization, validation, and rendering must stay in the pure layer
  so they are unit-testable without disk or config; only unit resolution and
  hydration touch the shell.
- **Forward compatibility.** The parser must tolerate additive optional fields
  (a later `mode`/`distribution`) so a v1 two-bound estimate stays valid under a
  future facet extension — unknown facet keys must not invalidate the table.
- **Coupling.** The graph-exposure contract is the only outward surface; keeping it
  policy-free is what prevents aggregation semantics from leaking into the facet
  and ossifying there.

## Hypotheses

- The entity engine's existing typed-facet-table seam (SPEC-004) is sufficient to
  carry `[estimate]` for any kind without per-kind schema work — adoption beyond the
  first wired kind (slice) is mechanical.
- Ordinary epistemic relations on the exposed edges are enough for tooling to infer
  estimate drivers; a dedicated `estimate_informed_by` relation is unnecessary in v1
  (PRD-014 OQ-5).
- Two bounds plus a project unit are sufficient input for any downstream
  aggregation/simulation policy; nothing those policies need must be persisted in
  the facet.

## Decisions

- **D1 — Component on the entity engine, not a new container.** The facet rides
  SPEC-004's shared materialiser and facet-table mechanism; it descends PRD-014 and
  parents to SPEC-004. It is not a kind, so it adds no `Kind` descriptor — only a
  facet model and the parse/hydrate wiring per adopting kind.
- **D2 — Normalize at the boundary to finite `f64`.** Integer/float authoring is a
  convenience; the internal model is uniformly finite `f64`, and non-finite values
  are rejected where they enter rather than carried inward.
- **D3 — One spec; raw graph-exposure folded in, aggregation deferred.** This spec
  covers the facet plus the *raw* per-node exposure (REQ-265's contract). Named
  aggregation policies, traversal-sensitivity, and simulation are PRD-014 non-goals
  and stay caller-side; a separate Cordage-aggregation spec is authored only if/when
  that work is greenlit (PRD-014 OQ-3, open).
- **D4 — Display classifies nothing by default.** Spread (ratio/width) is derived
  and shown only in verbose/review views; the default render emits no
  wide/risky/split verdict (PRD-014 principle).
- **D5 — Value is a sibling facet, not a sub-facet of Estimate.** The Value facet
  is a separate model (`ValueFacet`, one `f64` magnitude) with its own parse path
  and config section. It shares only the architectural pattern (leaf module,
  custom Deserialize, `#[serde(flatten)] _extra` forward-compat); no code is
  abstracted across the two.
