# CLI graph emitter and ego-view

## Context

Originates from IDE-043. RFC-001's thesis: doctrine's graph value is gated on
consumption surfaces. The semantic graph has exactly one rendering surface —
the web explorer (`map serve`, PRD-016) — and no CLI emission at all. An agent
or human in a terminal cannot see an entity's neighbourhood without starting a
web server.

Architecture facts (verified during scoping):

- The presentation-neutral read model exists in Rust: `CatalogGraph`
  (`src/catalog/graph.rs`), nodes indexed by key + flat edge list, with a thin
  JSON dump served at `/api/graph`.
- DOT serialization of the semantic view exists **only client-side**
  (`web/map/src/dot.ts`); focus/depth neighbourhood bounding is also
  frontend-side. Neither exists in Rust.
- `concept_map.rs` already emits DOT for the (separate) concept-map kind — a
  candidate DRY seam for shared DOT-writing helpers, to be assessed in
  `/design`.

PRD-016 §2 demotes *static graph file interchange for external tools* to
on-demand. This slice is distinguishable — an in-workflow, pipe-composable
consumption surface, per RFC-001 — but the PRD-016 boundary sentence should be
revisited at reconcile time.

## Scope & Objectives

One new CLI verb over the existing `CatalogGraph` read model:

```
doctrine graph [FOCUS] [OPTIONS]
```

- **Whole-corpus mode** (no FOCUS): emit the semantic graph projection.
- **Ego-view mode** (`graph SL-224`): focus entity + neighbourhood bounded by
  `--depth <N>` (default 1) — requires a Rust focus/depth BFS projection over
  `CatalogGraph` (new; port of the frontend behaviour).
- **Filters**, congruent with `relation list` vocabulary: `--kind <K>...`
  (repeatable node-kind filter), `--label <LABEL>` (edge label),
  `--include-memory` (off by default).
- **Formats**: `--format dot|json` (default `dot`). `dot` is a new Rust
  emitter (port of `web/map/src/dot.ts` semantics; share helpers with
  `concept_map.rs` where that is a genuine seam). `json` re-uses the existing
  `/api/graph` payload shape so CLI and web consume one contract.
- Composability is the product: `doctrine graph --format dot | gvpr … | dot -Tsvg`.

## Non-Goals

- `--render` / inline terminal display (kitty/sixel/viu) — follow-on.
- `mermaid` / `d2` output formats — follow-on.
- `--view actionability` and `--view coverage` altitudes — follow-on; the verb's
  option surface should not preclude them.
- Any change to the web explorer, its frontend DOT builder, or `/api/graph`.
- Computing or re-deriving priority/actionability (PRD-016: consume, never
  re-derive).
- Concept-map export — `concept-map export` already owns it.

## Summary

New `doctrine graph` verb: DOT/JSON emission of the semantic entity graph to
stdout, whole-corpus or focus+depth ego-view, with kind/label/memory filters —
the CLI consumption surface RFC-001 says the graph lacks.

## Risks, Assumptions, Open Questions

- **A1**: `CatalogGraph` hydration is cheap enough for a one-shot CLI verb (it
  already backs `map serve` startup and `/api/refresh`).
- **OQ-1**: whole-corpus DOT may be unusably dense — require a focus or filter,
  with `--all` override? Decide in `/design`.
- **OQ-2**: how much of `dot.ts` (styling, clustering, dark-theme colours —
  note mem_019ecf333d: dark-theme edge contrast gotcha) should the Rust
  emitter reproduce vs a leaner structural emission? Decide in `/design`.
- **R1**: divergence risk between TS and Rust DOT emitters — two renderings of
  one graph. Mitigation: keep the Rust emitter structural/minimal; do not aim
  for pixel parity.

## Verification / Closure Intent

- VT: unit tests over the focus/depth projection (bounding, filters) and DOT
  emission (valid DOT, expected nodes/edges for a fixture corpus).
- VT: `--format json` payload matches the `/api/graph` contract shape.
- VA: `doctrine graph <id> --format dot | dot -Tsvg` renders without graphviz
  errors on the real corpus (where `dot` is available).
- Done = verb shipped with tests green, `just gate` clean, PRD-016 boundary
  note raised at reconcile.

## Follow-Ups

- Inline terminal rendering (`--render`), mermaid/d2 formats, coverage /
  actionability views — tracked in IDE-043.
