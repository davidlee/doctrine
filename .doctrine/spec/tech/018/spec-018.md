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
the labels into semantic classes (composition, authorization, the work→canon
`references` axis refined by role, peer association, succession, and knowledge-record
epistemic edges).

## Responsibilities

### The relation model

A relation is an authored, **outbound-only** directed edge `(source, label, role?,
target)`:

- **source** — a numbered entity (`SL`, `SPEC`/`PRD`, governance `ADR`/`POL`/`STD`,
  `REQ`, `RV`, `REC`, backlog), identified by its canonical id.
- **label** — the relation kind (a `RelationLabel`), e.g. `governed_by`, `related`,
  `consumes`, `references`. Its wire spelling is `name()`.
- **role** — an optional closed `Role` (ADR-016) refining a label whose structure
  alone underspecifies intent. Present only for `references`
  (`{implements, originates_from, concerns}`); `None` elsewhere. The role, not the label,
  carries the target gate when present.
- **target** — a canonical ref to another entity, or free text for unvalidated
  labels.
- **degree** — an optional non-keyed `Degree {full, partial}` payload column, present
  only on the `fulfils` label (ADR-018), `None ≡ full`. Unlike `role` it keys no gate and
  is **excluded from edge identity** `(label, role, target)`; it records how much of the
  target a `fulfils` edge satisfies and never aggregates (two `partial` ≠ `full`).

Inbound is **never stored**. The reciprocal view ("what governs this ADR?", "what
supersedes this slice?") is derived from `in_edges` in the SL-046 graph (ADR-004);
the sole typed reverse is the `superseded_by` carve-out (ADR-004 §5), projected by
no reader. The on-disk storage shape (below) never reaches the graph layer —
`RelationEdge` normalises every edge before cordage, so uniform storage buys
writer/reader code simplicity, not graph correctness.

### The vocabulary: one code-authoritative source of truth

The legal-set vocabulary is the `RELATION_RULES` table in `src/relation.rs`,
keyed by **`(source ∈ sources, label, role)`** — a two-level closed grammar
(ADR-016): the label fixes the durable structural relation shape, and an optional
closed `Role` refines its *intent* where one label serves several. Each rule fixes
six axes — see the code for the authoritative set; this spec describes only their
*meaning*:

- **sources** — the source kinds that may author the label (a source-*set*, so one
  rule serves a label from several kinds without row explosion).
- **role** (`Option<Role>`) — the closed intent dimension. `None` for labels whose
  structure already fixes intent; `Some(Role)` for `references`, where
  `{implements, originates_from, concerns}` separate distinct intents under one label.
  `(source, label)` admits a fixed *set* of legal roles (`legal_roles`); each new
  intent is a code change, not a free-text value (cost #1). (`fulfils` is a distinct
  *label*, not a `references` role — its completion facet is the `degree` column, not a
  role; ADR-018.)
- **target** (`TargetSpec`) — `Kinds(…)` (a fixed legal target-kind set),
  `SameKind` (target kind equals source kind, e.g. governance `related`),
  `AnyNumbered`, or `Unvalidated` (free text). Keyed by `(source, label, role)` when
  a role is present (`references(implements) → {SPEC,PRD,REQ}`;
  `references(originates_from) → Kinds(BACKLOG + SL)`; `references(concerns) → AnyNumbered`),
  by `(source, label)` otherwise. Drives forward validation.
- **tier** — the storage shape (below).
- **link** (`LinkPolicy`) — whether the generic `link` verb admits the triple:
  `Writable`, `LifecycleOnly` (owned by a lifecycle transition, never plain-link —
  governance `supersedes`), or `TypedVerbOnly` (owned by a bespoke verb, e.g. `spec
  req add`).
- **inbound_name** — how the *derived* reciprocal renders on the target. Keyed
  `(label, role)` where a role is present (`references(implements)` → "implemented by",
  `references(originates_from)` → "originated from", `references(concerns)` → "concerned by"),
  `(label)` otherwise (`governed_by` → "governs", `fulfils` → "fulfilled by"); render-text
  only, and pinned
  `== name()` for every pre-existing label so legacy inbound output is unchanged.

This one table is the sole driver of **five consumers** — the `read_block` parser's
per-kind legality, the `link`/`unlink` writer dispatch, forward-edge validation, the
SL-046 reader's emitted edges, and cordage overlay allocation — held in lockstep by
an **exact-coverage** invariant test (not subset): per source kind the reader's
emitted `(label, role)` pairs equal the table's, and the overlay-backed label set
equals the resolvable graph labels (excluding the `Unvalidated` no-overlay labels). The
contract is outbound-only **by construction**: the table admits no inverse/derived
label, so `superseded_by` is structurally un-authorable as a relation.

### The tier partition

Relations are partitioned by storage shape, not flattened to one idiom:

- **Tier-1 — uniform `[[relation]]`.** Clean multi-ref edges (slice/backlog
  `references`; slice `supersedes`/`governed_by`; governance `related`; backlog
  `slices`/`drift`; spec `governed_by`/`consumes`) share one on-disk idiom: repeated
  `[[relation]] label=… role=… target=…` rows (the `role` key present only for
  role-bearing labels). One row per edge; `link` appends one, `unlink` removes the
  matching `(label, role, target)` triple.
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

- **Write (`link`) is strict.** A role-bearing label refuses a triple whose role is
  absent (`MissingRole`) or not in the label's legal set (`IllegalRole`); a roleless
  label refuses a stray role (`RoleNotApplicable`). The target gate then keys on the
  resolved `(source, label, role)` rule: for `Kinds`/`SameKind` targets the verb
  refuses unless the target both **resolves** to a real entity (`ensure_ref_resolves`)
  and **passes a legal-kind assertion** (`parse_canonical_ref(target).kind` is in the
  rule's kind set, or equals the source kind for `SameKind`) — existence alone is
  insufficient, since `ensure_ref_resolves` does not check target kind. Illegal
  triples and dangling numbered targets are hard-refused at write; `Unvalidated`
  targets accept free text.
- **Read / corpus `validate` is tolerant.** It *reports*, never rewrites:
  `[[relation]]` danglers that arise later (target deleted post-authoring),
  `read_block` `IllegalRow`s (hand-edited rows whose `(source,label)` is off-table,
  whose role is missing/illegal for a role-bearing label, or whose target-kind is
  outside the resolved rule), and the **supersession cross-check** —
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

### The `references` collapse and its dogfood migration

The work→canon label family — the noun-named `specs` (SL→`{SPEC,PRD}`) and the
standalone `requirements` label (SL→`REQ`) — folded onto the single `references`
label refined by role (ADR-016): the missing **verb** *is* the role.
`references(implements)` (SL → `{SPEC,PRD,REQ}`) absorbs `specs`/`requirements`;
`references(originates_from)` (SL → backlog; named `scoped_from` at SL-149, renamed at
SL-176) and `references(concerns)` (work → any numbered) absorb the mismapped `related`
rows that asserted scope or aboutness.
True symmetric peers stay on `related`, which does **not** fold — symmetry is
structural, so it remains its own label. `reviews` as a lightweight role was
dropped (it folds into `concerns`; heavyweight review keeps the first-class RV
`reviews` label).

The corpus was migrated **dogfood-only, no dual-read**: a one-shot out-of-band pass
(not a shipped CLI verb) re-censused the live corpus, applied the deterministic
`(source-kind, label, target-kind)` map, hand-dispositioned the ambiguous residue
(SL→SPEC `implements`-vs-`concerns`; every non-peer `related` row), then rewrote the
**whole** corpus in memory and applied it as a single atomic swap — the role-aware
parser and the rewritten corpus land in the **same commit**, so no commit holds code
and corpus in disagreement. The migration is recorded in
[migration-dispositions.md](../../../slice/149/migration-dispositions.md): 195 edges
(implements 93 · concerns 76 · scoped_from 14 · related-kept 12), with a per-row role
and rationale for every hand-judged edge, asserted by the role-assignment oracle.

### Finishing Axis B (SL-176 / ADR-018)

ADR-016 collapsed the work→**canon** half; the work→**backlog** half (`slices`, `drift`)
stayed standing until **SL-176** (ratified by **ADR-018**). The `slices` edge conflated
provenance, fulfilment, and completion; it is retired and split:

- **Provenance** — the `scoped_from` role is renamed **`originates_from`** in place
  (inbound "originated from") and widened to `{SL + backlog}` sources and
  `Kinds(BACKLOG + SL)` targets, subsuming the proposed `spawned_from`: a backlog item
  authors "born from SL", a slice "born from idea / sibling slice".
- **Fulfilment** — a new **`fulfils`** label (SL → backlog) carries the old "addressed by"
  reading as derived inbound "fulfilled by"; `backlog show` renders it through the same
  inbound machinery `inspect` uses (ADR-004-consistent).
- **Completion** — a non-keyed **`Degree {full, partial}`** column on `fulfils` (`None ≡
  full`, excluded from edge identity), the one place ADR-016 §2's derivable-not-relational
  law is partially reversed (completion is per-edge fact, not a status projection).

`drift` is untouched here (free-text escape hatch, deferred IMP-012/IDE-015). The priority
consequence of `fulfils` is a **value-burndown** post-pass (a backlog item's value reduced
by the value of the slices fulfilling it), not the old additive `slices`→optionality credit
(SPEC-001 / ADR-018). The migration record is
[migration-dispositions.md](../../../slice/176/migration-dispositions.md).

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
- **Specs to rewire (follow-up).** **SPEC-005** (ADR entity surface — its
  reserved-but-inert `[relationships]` seam), **SPEC-006** (spec composition
  machinery), and **SPEC-016** (governance kinds) currently each tell part of the
  relation story; they are to be rewired to *reference* this contract rather than
  re-telling it, now that the contract is proven in code. Not in SL-149's scope —
  carried as a follow-up.
