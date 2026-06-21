# SPEC-018: Cross-corpus relation contract

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The relation contract is the authored-edge substrate beneath the graph: it fixes
**which kinds may link to what, the edge semantics, the on-disk storage shape, the
write seam, and validation policy** for cross-corpus relations. It realises
**PRD-011** (graph-derived priority and actionability) on the capture side — SL-046
shipped the *reader* (`outbound_for` → `RelationEdge{label,target}`, derived
inbound) and SL-047 the *rank*; this component is the *authored relations* those
layers read. It sits under the graph engine container (**SPEC-001**) as the layer
the registry adapter scans, and is governed by **ADR-010** (the relation contract)
composing with **ADR-004** (relations stored outbound-only; reciprocity derived).

The forcing decision — the model, tiers, vocabulary location, validation policy,
and outbound-only composition — was taken in **ADR-010 (accepted)**. This spec
*describes* that contract semantically and **points at its authoritative
enumeration in code** (`RELATION_RULES`, `src/relation.rs`); per the storage rule
it never transcribes the vocabulary, which would duplicate queried data into prose.

A companion reference — [relation-vocabulary.md](relation-vocabulary.md) — groups
the 17 labels into semantic classes (composition, authorization, work→artefact,
peer association, succession, and the epistemic gap for knowledge records).

## Responsibilities

### The relation model

A relation is an authored, **outbound-only** directed edge `(source, label,
target)`:

- **source** — a numbered entity (`SL`, `SPEC`/`PRD`, governance `ADR`/`POL`/`STD`,
  `REQ`, `RV`, `REC`, backlog), identified by its canonical id.
- **label** — the relation kind (a `RelationLabel`), e.g. `governed_by`, `related`,
  `consumes`, `specs`. Its wire spelling is `name()`.
- **target** — a canonical ref to another entity, or free text for unvalidated
  labels.

Inbound is **never stored**. The reciprocal view ("what governs this ADR?", "what
supersedes this slice?") is derived from `in_edges` in the SL-046 graph (ADR-004);
the sole typed reverse is the `superseded_by` carve-out (ADR-004 §5), projected by
no reader. The on-disk storage shape (below) never reaches the graph layer —
`RelationEdge` normalises every edge before cordage, so uniform storage buys
writer/reader code simplicity, not graph correctness.

### The vocabulary: one code-authoritative source of truth

The legal-set vocabulary is the `RELATION_RULES` table in `src/relation.rs`,
keyed by **`(source ∈ sources, label)`**. Each rule fixes five axes — see the code
for the authoritative set; this spec describes only their *meaning*:

- **sources** — the source kinds that may author the label (a source-*set*, so one
  rule serves `specs` from both slice and backlog without row explosion).
- **target** (`TargetSpec`) — `Kinds(…)` (a fixed legal target-kind set),
  `SameKind` (target kind equals source kind, e.g. governance `related`),
  `AnyNumbered`, or `Unvalidated` (free text). Drives forward validation.
- **tier** — the storage shape (below).
- **link** (`LinkPolicy`) — whether the generic `link` verb admits the triple:
  `Writable`, `LifecycleOnly` (owned by a lifecycle transition, never plain-link —
  governance `supersedes`), or `TypedVerbOnly` (owned by a bespoke verb, e.g. `spec
  req add`).
- **inbound_name** — how the *derived* reciprocal renders on the target
  (`governed_by` → "governs"); render-text only, and pinned `== name()` for every
  pre-existing label so legacy inbound output is unchanged.

This one table is the sole driver of **five consumers** — the `read_block` parser's
per-kind legality, the `link`/`unlink` writer dispatch, forward-edge validation, the
SL-046 reader's emitted edges, and cordage overlay allocation — held in lockstep by
an **exact-coverage** invariant test (not subset): per source kind the reader's
emitted labels equal the table's labels, and the overlay-backed label set equals the
resolvable graph labels (excluding the `Unvalidated` no-overlay labels). The
contract is outbound-only **by construction**: the table admits no inverse/derived
label, so `superseded_by` is structurally un-authorable as a relation.

### The tier partition

Relations are partitioned by storage shape, not flattened to one idiom:

- **Tier-1 — uniform `[[relation]]`.** Clean multi-ref edges (slice `specs`/
  `requirements`/`supersedes`/`governed_by`; governance `related`; backlog
  `slices`/`specs`/`drift`; spec `governed_by`/`consumes`) share one on-disk idiom:
  repeated `[[relation]] label=… target=…` rows. One row per edge; `link` appends
  one, `unlink` removes one.
- **Tier-2 — kept typed.** Constrained-arity or payload-bearing edges keep their
  bespoke storage and guarantees: spec lineage (`descends_from`/`parent`, arity ≤1),
  `members` (members.toml), `interactions` (free-text + payload), review `[target]`,
  rec `owning_slice`, backlog `needs`/`after`/`triggers`. Genericising these would
  flatten their guarantees (ADR-010 rejected this).
- **Tier-3 — free text.** Unvalidated targets (`decision_ref`) carry as-is.

**Two storage invariants** (F1) make tier-1 safe to cohabit with typed leftovers:
typed tables must *precede* every `[[relation]]` array (bare keys after an
array-of-tables header bind to the last table — silent corruption), and the writer
appends arrays only at EOF. A hand-edit that violates the ordering is an
`IllegalRow`-class finding for `validate`, never a silent splice.

**Governance supersession is excluded from tier-1 migration** (OD-3): `supersedes`
and `superseded_by` stay typed *together* — the forward edge is meaningless to move
while its sanctioned reverse carve-out has no transactional owning verb (that verb
is IMP-006's cross-kind lifecycle axis; building it gov-only here would be parallel
implementation). Only governance `related` migrates and `supersedes` (SL-095).
`tags` is classification, not a
relation; it moved to root-level uniform storage in SL-136.

### Validation policy: write-strict, read-tolerant

- **Write (`link`) is strict.** For `Kinds`/`SameKind` targets the verb refuses
  unless the target both **resolves** to a real entity (`ensure_ref_resolves`) and
  **passes a legal-kind assertion** (`parse_canonical_ref(target).kind` is in the
  rule's kind set, or equals the source kind for `SameKind`) — existence alone is
  insufficient, since `ensure_ref_resolves` does not check target kind. Illegal
  triples and dangling numbered targets are hard-refused at write; `Unvalidated`
  targets accept free text.
- **Read / corpus `validate` is tolerant.** It *reports*, never rewrites:
  `[[relation]]` danglers that arise later (target deleted post-authoring),
  `read_block` `IllegalRow`s (hand-edited rows whose `(source,label)` is off-table
  or whose target-kind is outside the rule), and the **supersession cross-check** —
  where a governance entity's stored `superseded_by` disagrees with the reciprocal
  derived from `supersedes` in-edges. The cross-check reads the typed field directly
  (the generic read seam excludes `superseded_by` by construction) and may surface
  pre-existing hand-authored drift, which is the point.

### The write seam

The cross-kind writer is the uniform `link`/`unlink` verb over generic
`append_edge`/`remove_edge` (edit-preserving, idempotent), gated by the table's
`LinkPolicy`. Governance `related` and the new `governed_by`/`consumes` edges become
*authorable* rather than hand-edited inert TOML. Tier-2/3 edges keep their bespoke
owning verbs.

## Concerns

- **Behaviour preservation across the migration.** The tier-1 storage move is gated
  by before/after byte-identical render goldens (`inspect` / `*-show` / `show
  --json`) **and** a storage-level post-check — render goldens alone are not a
  sufficient oracle, because the render path launders on-disk row order (the
  `inspect` BTreeMap regroup; canonical-ordered `format_show`). The SL-046 reader
  must emit the same edges for already-authored relations after the migration.
- **Determinism.** No clock/RNG/`HashMap` iteration order anywhere on the relation
  path — BTree ordering only, and one canonical label order (`RELATION_RULES`
  declaration order) shared by every render surface.
- **Layering (ADR-001).** Vocabulary and the generic seam are leaf/engine
  (`relation.rs`); the `link`/`unlink` verb is command. cordage stays unaware of
  doctrine relation vocabulary (overlay allocation is table-derived in the adapter).

## Hypotheses

- **Storage uniformity ≠ behaviour uniformity.** A generic parser must still enforce
  per-kind legality (a slice cannot author `related`; a backlog item cannot author
  `governed_by`), so `read_block` is source-kind-aware and routes illegal rows to
  validation findings rather than live edges.
- **Dogfood-only, no client back-compat.** Parsers cut hard to `[[relation]]` (no
  dual-read, no shipped migrate verb); the one-time corpus rewrite is an out-of-band
  deterministic pass, verified by round-trip `show` + `validate` before commit.

## Decisions

Project-global relation decisions live in **ADR-010** (the contract) and **ADR-004**
(outbound-only); this spec does not restate them. Decisions local to the spec's
framing:

- **Authority by pointer, not transcription.** The vocabulary's legal set lives in
  `RELATION_RULES` (code); this spec names the axes and their meaning and points at
  the code, honouring the storage rule. Any enumeration here would be derived data
  in prose.
- **References to consolidate (PHASE-06).** **SPEC-005** (ADR entity surface — its
  reserved-but-inert `[relationships]` seam), **SPEC-006** (spec composition
  machinery), and **SPEC-016** (governance kinds) currently each tell part of the
  relation story; they are to be rewired to *reference* this contract rather than
  re-telling it, once the contract is proven in code.
