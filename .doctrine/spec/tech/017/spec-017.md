# SPEC-017: Tech-spec spine

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The tech-spec spine is the set of flat, hand-authored fields that place a tech spec
in the corpus tree and bind it to code: `descends_from`, `parent`, `c4_level`, and
the repeatable `[[source]]` anchors — plus the tech-only `interactions.toml`
peer-edge table that runs alongside them as the distinct peering axis. It is a
component of the spec composition machinery (SPEC-006); that container owns *how* a
spec is composed, reassembled, and FK-validated, and this component details the
spine *capability* those mechanisms ride on. Everything shared — the two subtypes,
the requirement-as-peer model, the `spec show` reassembly, and the `spec validate`
FK-integrity pass — lives in the parent and is cited here, never restated. This
component carries only what is specific to the spine: its field shapes and
serde defaults, the closed C4 vocabulary, the outbound-only discipline, the source
anchor model, the kind/Some-gated render contract, and the registry edge harvest
that feeds validation.

## Responsibilities

Mirrors the structured `responsibilities` list: define the tech-only flat spine
fields and their serde defaults; fix the closed `c4_level` set; carry single-valued
outbound `descends_from`/`parent`; model the repeatable `[[source]]` anchor; render
the spine kind-gated and Some-gated; carry the `interactions.toml` peer-edge axis;
and harvest spine refs and edges into the registry for the parent's `validate`.

### The spine fields and their serde defaults

The spine is four optional flat fields on the parsed `Spec`: `descends_from:
Option<String>`, `parent: Option<String>`, `c4_level: Option<C4Level>`, and
`sources: Vec<Source>`. Each is `#[serde(default)]`, so a product spec or an
unfilled tech spec round-trips with the scalars `None` and the anchor list empty —
the at-rest default. There is no CLI flag for any spine field; `spec new` takes
only subtype/title/slug, and the spine is hand-edited TOML the engine round-trips
but never generates. The fields are tech-only: a product spec carrying any of them
is the parent container's invalid-kind concern, not a tolerated extra.

### The closed C4 level

`c4_level` is an `Option<C4Level>` over the closed set **context · container ·
component · code**, kebab-serded. It is the altitude axis: hand-authored specs
normally stop at container/component, code is exceptional. The set is closed so an
out-of-vocab level fails to parse rather than being silently carried; it is optional
because a spec may be authored before its altitude is fixed.

### Outbound descent and decomposition

`descends_from` is the single-valued cross-family link from a tech spec to the
`PRD-NNN` capability it realises; `parent` is the single-valued `SPEC-NNN`
decomposition parent. Both are scalar `Option<String>` — single-valued by type, so
a second value is a parse error, not a list. Both are **outbound-only** (ADR-004):
the reciprocal views — a PRD's realising specs, a parent's children — are derived
on read, never authored on both sides. Render reflects this: children are never
emitted, only the outbound edge.

### The source anchor model

A `[[source]]` is `{language, identifier, module?}` — a language tag, a code
identifier (a path), and an optional finer module path — parsed via `#[serde(rename
= "source")]` into `sources: Vec<Source>`, the repeatable list of code locations the
spec governs. The anchor is descriptive, not load-bearing: liveness is not checked
(an anchor is not proof the spec is current), so a stale anchor ships silently and
authoring must verify paths against `src/` by hand.

### The kind- and Some-gated render contract

`spec show` renders the spine guarded twice. The `descends from` / `parent` lines
emit only when `kind == Tech` *and* the field is `Some` — the kind gate stops a
(hard-invalid) spine field on a product spec from being legitimised by render, the
Some gate keeps product and unfilled-tech output byte-identical. `c4 level` emits
when present; the `sources:` block emits only when the anchor list is non-empty,
each anchor rendered `<language> <identifier> (<module>)` with the parenthesised
module omitted when absent.

### The interaction peer-edge axis

The tech-only `interactions.toml` carries `[[edge]]` rows of `{target, type,
notes?}` — outbound spec→spec peer relations (`uses`/`calls`/free-text `type`),
parsed under the `edge` key (not `[[interaction]]`). A missing file yields zero
edges, so a product spec (no file) and a tech spec with an empty seed both resolve
to `[]` uniformly. This is the **peering** axis, deliberately distinct from `parent`
containment: a peering is never encoded as decomposition and vice versa. Edges are
outbound-only; the reverse direction is derived.

### Registry harvest for validation

Parsing a spec into the registry pushes its `descends_from` onto `DescentEdge`, its
`parent` onto `ParentEdge` (canonicalised), and its interaction edges onto
`InteractionEdge`, so the parent container's `spec validate` can FK-check every
spine target. A duplicate-key or array-valued `parent` is classified as a named
`second_parent` finding (carried, not propagated as a raw parse error) before it can
reach the registry. The integrity pass itself is the parent's; this component only
supplies the harvested edges and the `second_parent` classification.

## Concerns

- **Anchors are not currency.** `[[source]]` liveness is never validated, so a
  renamed or deleted path leaves a stale anchor that `validate` cannot catch — the
  anchor is descriptive, and freshness is an authoring-time discipline.
- **Single-valued by type, not by validation.** `descends_from` and `parent` are
  scalars, so a second value is a parse failure; the `second_parent` classifier
  exists precisely to turn that low-level toml error into a legible finding rather
  than an opaque parse abort.
- **Containment-vs-peer discipline is unenforced.** Nothing stops a `parent` that
  *should* have been an interaction edge (or the reverse); keeping the two axes
  distinct is an authoring obligation, not a mechanical guard.

## Hypotheses

- **The spine needs no verb of its own.** Beyond the field shapes, the C4
  vocabulary, the anchor model, the render gates, and the registry harvest, every
  behaviour the spine relies on — scaffold, reassembly, FK-integrity — is the parent
  container's. The component stays thin without becoming a stub because the spine
  fields, their serde and render contracts, and the interaction axis are real
  kind-specific surface.

## Decisions

- **D1 — spine fields are authored TOML, never generated.** No CLI flag sets a
  spine field; the engine round-trips the hand-edited TOML and `spec validate` is the
  only gate, consistent with the storage rule.
- **D2 — single-valued scalars, outbound-only.** `descends_from` and `parent` are
  `Option<String>`, not lists; the reciprocal direction is derived (ADR-004), and a
  multi-value attempt surfaces as the `second_parent` classification.
- **D3 — closed C4 set, open everywhere it must be.** `c4_level` is a closed
  kebab-serded enum so out-of-vocab levels fail to parse, while remaining optional
  so altitude can be fixed after the spec is first scaffolded.
- **D4 — peering is a separate axis from containment.** Interaction `[[edge]]` rows
  carry `uses`/`calls` peers in `interactions.toml`; `parent` carries decomposition.
  The two are never substituted, and an absent interactions file is zero edges, not
  an error.
