# Cascade trait-set selection

## Context

SL-186 delivered the prompt-cascade resolver with a **single-valued** model
axis: `ContextVector.model: Option<String>` (hymns.rs:202), one pattern per
`Selector` (hymns.rs:149). The design conversation specified the model band as
a **composable, user-definable classification space** of orthogonal trait axes
(`adherence/*`, `capability/code/*`, `capability/reasoning/*`); design.md's
compaction flattened that to a `vendor/name` path and the engine encodes the
flattened version. RFC-013 recovers the nuance; SPEC-023 (now active) specifies
the target algebra. This slice is the **conformance fix**: it makes the
delivered engine match SPEC-023's forward-intent — framed as SL-186
under-delivering its own composable-category design, not new scope.

Forcing function: SL-191 authors trait-keyed worker hymns
(`model/adherence/low.md` ∧ `model/capability/code/high.md` composing for one
worker) and cannot express them on the single-key engine. SL-191 `after:`s this
slice.

## Scope & Objectives

Implement SPEC-023 FR-004, FR-005, FR-007 (+ the FR-009 CLI arity change):

1. **Set-valued context model axis (FR-004).** `ContextVector.model` becomes a
   set of trait keys; a model pattern matches by **membership** (matches any
   member). An agent is a set of points; all its declared trait guidance
   composes in one resolve.
2. **Selector conjunctive model pattern-set (FR-005).** `Selector.model`
   becomes a set of patterns; **every** pinned pattern must match some context
   key — intersection targeting ("smart AND loose only") with the selector
   still a pure conjunction. Sidecar TOML accepts a list; path-derived
   selectors stay single-pattern.
3. **Root-wise normalized model specificity (FR-007).** Model-axis specificity
   compares pinned patterns as ordered `(root, depth)` pairs, lexicographic:
   same-root stays depth-ordered; intersections outrank their factors;
   cross-tree ordering is root-name-alpha — deterministic and stable under
   taxonomy deepening. Precedence-key total order preserved (specificity stays
   a context-free function of the snippet).
4. **Repeatable `--model` (FR-009 delta).** `prompt resolve`/`explain` accept
   repeated `--model` occurrences building the context trait set.

Existing single-model behaviour must degrade to the singleton case: current
goldens pass with singleton sets (modulo the specificity type change where the
D3 tuple's primary component generalizes).

## Non-Goals

- **No hymn content.** Trait-keyed worker/adherence hymns, def frontmatter
  trait declarations, and the bake widening are SL-191.
- **No new bands.** Trait categories are labels within the `model` band;
  INV-1's closed registry untouched.
- **No grammar-OR.** Disjunction stays in classification (SPEC-023 NF-003 /
  D4); nothing here adds boolean selector grammar.
- **No classification/mapping machinery.** How an agent comes to carry trait
  keys (def frontmatter, spawner, env heuristics) is outside (SPEC-023 D5 /
  OQ-1).
- **No delivery changes** beyond CLI arity — boot/session-start/onboard wiring
  (SL-187 surfaces) untouched.

## Summary

Bounded engine change in `src/hymns.rs` + `src/commands/prompt.rs`: context
model axis set-valued (membership match), selector model axis
conjunctive-set-valued (intersection targeting), root-wise normalized
specificity, repeatable `--model`. Loader sidecar accepts a pattern list.
Specificity/precedence remain a total order; band registry, seal, replaces,
provenance untouched. Delivers the composition SL-191's authoring depends on.

## Verification / closure intent

- Engine goldens: membership match (multi-key context, single-pattern
  selector); intersection match (multi-pattern selector hits only agents
  carrying all); normalized specificity table (same-root depth, intersection >
  factor, cross-root alpha-stability under deepening); singleton degeneration
  (existing behaviour).
- E2E: `prompt resolve --role worker --model adherence/low --model
  capability/code/high` composes both trait snippets; `explain` traces the
  multi-key match.
- Behaviour-preservation: existing resolver/loader/e2e suites green with
  singleton contexts (any golden churn confined to the specificity type
  generalization, reviewed by intent).

## Follow-Ups

- SL-191 rides this: trait hymns + def trait-set declaration + bake widening.
- SPEC-023 OQ-3 (required-trait `prompt check` lint) — candidate here or
  SL-191; decide in `/design`.
- SPEC-023 D4 (disjunction-via-classification) ADR graduation — separate,
  governance-side.
