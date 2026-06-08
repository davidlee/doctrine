# Technical-spec system support: descent, decomposition & integrity

## Context

SL-015 shipped the spec entity with a `tech` subtype: identity, requirements as
peer entities, membership labels, on-demand `spec show` reassembly, and the
`spec validate` FK gate (`mem.system.spec.composition-seam`). It also landed the
tech-only flat fields the *how* needs at rest: `c4_level` (closed
context|container|component|code enum, rendered) and `[[source]]` code anchors
(language / identifier / optional module, rendered). What it did **not** ship is
the relational spine PRD-012 v1 requires: a spec has no way to descend from the
product capability it realises, no way to decompose into a single-parent tree,
and the registry runs no decomposition integrity (cycle detection was explicitly
"deferred to the feature DAG").

PRD-012 ("Technical Specifications") settled the v1 surface: descent, C4 level,
single-parent acyclic decomposition, typed peer interactions, code anchors, and
lineage-via-supersession — with the importer (OQ-2) and dedicated transform
verbs (OQ-4) reserved. SL-021 (tech-spec corpus backfill) named this work as its
own prereq slice: decomposition as a *first-class structured field* (a
single-valued outbound `parent` FK under ADR-004 outbound-only), **not** a
free-text `interactions` edge — `interactions` stays for peer relations
(`uses`/`calls`). This slice builds that spine so SL-021 can author against a
complete tech surface.

This is the entity-model + integrity delta only — and a deliberately narrowed
one: supersession lineage support is deferred (see Non-Goals / Follow-Ups). The
folder hoist (`.doctrine/spec/{product,tech}/` → top-level) and the corpus
content are separate work (an ADR/migration and SL-021 respectively).

## Scope & Objectives

Close the gap between the SL-015 tech surface and PRD-012 v1, requirement by
requirement. Build only what is missing; reuse the shipped seams unchanged
(behaviour-preservation gate — the SL-015 suites stay green). All three relations
are **single-valued outbound scalar fields** on `spec-NNN.toml` (the `c4_level`
precedent), hand-edited, with `spec validate` as the integrity gate and `spec
show` rendering them outbound-only (ADR-004 §3). No new producer verb, no new
edge-table file.

1. **Cross-family descent (REQ-082 / FR-002).** Add a tech-only outbound scalar
   `descends_from = "PRD-NNN"` naming the product capability the spec realises,
   storing the target's durable peer id only — never a compound or owner-qualified
   key. The field name is **`descends_from`**, not `realises` (which overclaims:
   code realises intent, a spec describes the *how*). `spec validate` confirms the
   target resolves to an existing **product** spec; `spec show` renders it. The
   spec does not restate product intent — the field is a pointer, not prose.

2. **Single-parent decomposition (REQ-083 / FR-003).** Add a single-valued
   outbound `parent = "SPEC-NNN"` FK on a tech spec, stored once on the child
   (ADR-004 §1). The reciprocal (a parent's children) is **derived**, never stored
   — and per ADR-004 §3 it is the registry/`inspect` surface's to compute, **not**
   `spec show`'s (which stays outbound-only). `spec validate` confirms the parent
   resolves to an existing tech spec (dangling-parent is a hard finding).

3. **Decomposition integrity (REQ-087 / NF-001).** `spec validate` enforces the
   tree: a self-parent and any cycle in the parent chain are hard findings
   returning non-zero. A second parent is structurally precluded — a scalar
   `parent` key cannot appear twice in one TOML doc (duplicate-key parse error).
   This adds the parent-chain cycle detection the registry deferred, kept local to
   decomposition (no premature feature-DAG).

4. **Peer-interaction target-kind correctness (REQ-084 / FR-004).** Distinguish
   an *invalid target kind* (a peer `interactions` edge pointing at a product
   spec — the target exists but the edge type is wrong) from a *dangling
   reference* (target id resolves to nothing). Today a `PRD-*` interaction target
   is merely "absent from the tech_specs set" and reported as dangling; the
   narrowed PRD-012 §6 requires the kind distinction. Needs a `product_specs` set
   in the registry (which the descent check needs too).

## Non-Goals

- **Supersession system support (REQ-086 / FR-006) — deferred.** v1 keeps the
  lifecycle as shipped (the SL-015 `status` enum: `superseded` / `deprecated`,
  hand-edited). No `superseded_by` lineage field, no `spec supersede` verb, no
  orphan-on-superseded-parent integrity. The ADR-004 §5 carve-out (co-write
  `superseded_by` during the atomic status transition) makes this naturally a
  *verb*, distinct from the static hand-edited fields above; it is pulled out so
  it can be backlogged properly once SL-020's backlog entity lands. (A `parent`
  FK that resolves to nothing is still caught — that is the FR-003 dangling-parent
  check, not supersession.)
- **Derived children / reverse-descent view — deferred.** Per ADR-004 §3 the
  inbound views ("a parent's children", "which tech specs descend from this PRD")
  belong to the registry-backed `inspect`/survey surface, not the sync-free
  reader. v1 computes the parent→child inversion only *internally* for cycle
  detection; no user-facing children view ships. Follow-up.
- **Importer / hand↔import convergence code (REQ-088 / NF-002).** The importer's
  source and shape are unresolved (OQ-2). The code anchor (`[[source]]`) is
  already the single convergence seam; v1 builds no import path. The convergence
  requirement constrains a future importer, not this slice — satisfied-by-design
  (one anchor seam), no code.
- **Dedicated transform verbs (OQ-4) & Theseus identity (OQ-3).** Merge/split
  operations, automatic child re-parenting, and the in-place-evolution-vs-supersede
  threshold are reserved.
- **Drift ledger.** Recording a spec↔code mismatch is the drift capability's
  surface. This slice only keeps the anchor *data* a drift pass reads present and
  well-formed; it creates no drift record and resolves no anchor against source.
- **Evergreen-altitude enforcement (REQ-089 / NF-003).** Holding specs at durable
  architectural altitude is an authoring discipline enforced in `/design` and
  `spec-tech` SKILL guidance, not a code gate here.
- **C4 level & code anchors (REQ-081 / REQ-085).** Already shipped in SL-015;
  touched here only insofar as render/validate already handle them — not re-built.
- **Corpus content & folder hoist.** SL-021 and a separate ADR/migration.

## Affected surface

- `src/spec.rs` — `Spec` struct gains `descends_from: Option<String>` and
  `parent: Option<String>` (parse + `render()` outbound lines). Touches the parse
  / render seams only; existing fields unchanged. `build_registry` also collects
  product-spec ids and the two new edge kinds.
- `src/registry.rs` — gains a `product_specs: BTreeSet<String>` id set, parent
  edges, and descent edges; new pure checks: dangling/invalid-kind descent,
  dangling parent, self-parent, parent-chain cycle, and the interaction
  invalid-kind-vs-dangling split. Checks stay direct set-membership +
  one local chain-walk; no generic edge framework.
- `install/templates/spec-tech.toml` — document `descends_from` and `parent` in
  the scaffold comment block (mirrors the `c4_level` / `[[source]]` comments).
  Embedded template — heed the rust-embed re-embed footgun.
- Tests in `src/spec.rs` / `src/registry.rs` and any e2e harness covering
  `validate` / `show`.

## Risks, assumptions, open questions

- **Cycle detection placement.** The registry scan currently builds id sets +
  edge lists with no graph traversal; the parent-chain walk is the first
  DAG-shaped check. Keep it local to decomposition — do not generalise into a
  "feature DAG" prematurely.
- **`product_specs` set additive only.** The registry deliberately materialised
  no product id set ("no check resolves against one"). Descent + interaction-kind
  now need one; adding it must not perturb the existing four checks
  (behaviour-preservation — the SL-015 registry suite stays green unedited).
- **Descent is tech-only (resolved: warn, leave open).** Product specs do not
  descend in v1; `descends_from` on a product spec is **a soft `validate`
  warning** (printed, exit zero), not a hard finding and not silently ignored.
  This requires `validate` to gain a minimal **severity split** (hard findings
  bail non-zero; warnings are advisory) — the one structural addition beyond
  additive checks. Whether product specs should gain their own hierarchy/descent
  long-term is an **open question**, left undesigned — see Follow-Ups.
- **Behaviour-preservation.** The SL-015 spec/registry suites are the proof the
  shared machinery is unchanged — they must stay green unedited.
- **Assumption:** `c4_level` enum, `[[source]]` shape, and the membership/label
  seam are final for the duration (PRD-012 settled).
- **Assumption:** PRD-012 v1 narrowing (importer + transform verbs reserved)
  holds; no expansion mid-slice.

## Verification / closure intent

- Each in-scope requirement traces to a test: `descends_from` stored / rendered /
  validated against the product set (dangling and invalid-kind both flagged);
  `parent` stored-once and validated against the tech set; self-parent + cycle
  rejected non-zero; a duplicate `parent` key fails to parse; a `PRD-*` peer
  interaction reported as invalid-kind, not dangling.
- `doctrine spec validate` is green on a well-formed tech corpus and non-zero on
  each crafted violation; `spec show` reassembles a tech spec with `descends_from`,
  `parent`, peers, and anchors as one readable whole (outbound only — no children).
- SL-015 spec/registry suites pass unchanged (behaviour-preservation gate) —
  except the deliberate REQ-084 test rewrite and the mechanical `None, None`
  additions to `spec.rs` `Spec { … }` test constructors (no assertion changes
  value; `Spec` derives no `Default`).
- `just check` green; clippy zero warnings; storage rule honoured (structured
  relations in TOML, no derived data — children, reverse view — persisted).
- SL-021 is unblocked: the tech surface is complete enough to backfill against.

## Summary

## Follow-Ups

- **Supersession system support** (REQ-086 / FR-006). `superseded_by` lineage
  under the ADR-004 §5 carve-out, a `spec supersede` verb (atomic status-flip +
  reverse co-write), and orphan-on-superseded-parent integrity. Backlog once
  SL-020's backlog entity lands.
- **Derived children / reverse-descent view** on the registry-backed
  `inspect`/survey surface (ADR-004 §3). Surfaces "a parent's children" and "which
  tech specs realise this PRD" — the legibility PRD-012's success measures call
  for, which the sync-free `spec show` deliberately does not provide.
- **`descends_from` ↔ `realises` prose reconciliation in PRD-012/REQ-082** (if the
  user wants the requirement title's "realises" wording revisited — PRD-012
  territory, not SL-022).
- **Product-spec hierarchy / descent (open question).** v1 warns on
  `descends_from` placed on a product spec but does not forbid the *concept* of
  product-spec hierarchy. There is a plausible future case for decomposition among
  product specs; left undesigned deliberately. Revisit before hardening the
  warning into an error or extending decomposition cross-family.
