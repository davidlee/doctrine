# Entity model — consolidation direction

**Status: direction, deferred. No action now.** This is the umbrella the
per-entity notes ([slices-spec](slices-spec.md), [spec-entity-spec](spec-entity-spec.md),
[drift-spec](drift-spec.md), [reservation-spec](reservation-spec.md),
[relation-index](relation-index.md)) hang off. It records the *target* doctrine
model and adjudicates a critique of spec-driver's schema. It does **not** change
the build roadmap (§ What this does not change) and starts nothing new.

## Thesis

Spec-driver's schema is an accumulation of local fixes, not a target
architecture: two validation layers (frontmatter + embedded fenced YAML blocks,
each self-versioned), drift ledgers omitted from the registered surface, and
per-kind status dialects that have drifted from their own enums. doctrine is
greenfield, so it adopts the *shape* the per-entity notes already converge on:

**Model a small set of durable authored entities; attach typed facets and tables
to them. Do not model every file, block, phase, or index row as a first-class
kind.** This is the same move slices-spec made (structured metadata → sister TOML,
body → pure prose) generalised across the whole model.

## The storage rule (one rule everywhere)

```text
<entity-dir>/
  <entity>-<id>.toml   # identity, lifecycle, owners, summary, typed references
  <entity>-<id>.md     # prose only
  <facet>.toml         # flat structured rows (arrays-of-tables)
  <facet>.md           # prose keyed by row id — only when a facet needs narrative
```

Already in force for slices; the spec decomposition is the same rule at its
hairiest (spec-entity-spec § The decomposition). Consequences, all already
argued in the notes:

- **No embedded fenced YAML blocks.** The defect is not YAML-vs-TOML; it is
  embedding a queried registry in a hand-edited document (spec-entity-spec
  § Diagnosis). The registry reads tiny facet TOMLs, never markdown bodies
  (relation-index).
- **Prose lifts out, testable lists stay structured** (spec-entity-spec § The
  decomposition, B3): `acceptance_criteria`/`success_criteria` are rows, not prose.

## Templates are defaults; TOML is owned

The split has a second edge — who owns the *format*:

- **Prose templates (and, later, skills) are sensible defaults, not contracts.**
  Users may re-head, restructure, or delete sections of a scaffolded `.md`. The
  tooling must therefore **never parse prose structure or depend on its
  headings** — a prose template is a write-once scaffold, applied by token
  substitution only (a missing/renamed token is a harmless no-op, not an error).
  Templates can only substitute values on hand at scaffold time.
- **TOML facets are owned, locked formats.** The tooling reads, queries, and
  round-trips them (edit-preserving append, fixed schema); their shape is not the
  user's to restructure.

Corollary — and this is just the storage rule from the tooling-contract angle:
**anything the tooling must read or query lives in TOML, never prose.** It is why
a design doc's queryable data (date, key files, governance refs, approval) belongs
in a sister facet, not its markdown body (slice-003 design.md D5).

## Entity vs facet taxonomy

The [glossary](glossary.md) groups already prefigure this. Target consolidation —
**fewer entity kinds, more facets** (adjudicated, see § Adjudication):

| Durable entity | Subsumes | Facets / notes |
|---|---|---|
| `spec` family | product / tech | one model, **per-subtype folders + facet sets** (spec-entity-spec § Spec identity). A requirement is a **peer entity** (`REQ-NNN`), membered via `members.toml`; capabilities/coverage are deferred facets. (Revision is **not** a spec subtype — ADR-013 makes it a standalone change-axis kind; see § Adjudication.) |
| `slice` | delta, most of design-revision, parts of plan | the change contract (slices-spec). Design/IP/phase land as siblings/facets (slice-003, IP+phases slice). |
| `phase` / run | executable phase sheet only | keep only if resumable multi-agent execution needs it; else a slice facet. Carries mutable runtime state — different treatment (spec-entity-spec § Design-data vs runtime-state). |
| `audit` | audit + findings + coverage result | findings are rows; evidence stays prose/files. |
| `backlog_item` | issue, problem, risk, improvement, chore, idea | one kind + `item_kind`, not six schemas; risk gets extra facet fields. |
| `decision` / governance | policy, standard, ADR | one kind + `doc_kind`, or ADR separate if decision semantics earn it. |
| runtime state | workflow.*, session, review-orchestration | **not authored design schema** — see § Runtime state. |

## Identity and references

- **Canonical string id externally, numeric internally.** `id = "SPEC-110"`,
  `number = 110`. Cross-entity references use the target's **durable peer id**
  (`REQ-007`, `SPEC-110`) — every entity is addressable on its own, so there is no
  compound/owner-qualified key. Display labels like a requirement's `FR-001` /
  `NF-001` are **per-membership labels carried on the edge** (a spec's
  `members.toml` row), not identities (spec-entity-spec § Identity rules).
- **Edges.** A generic edge table carries payload-free links, targeting durable
  peer ids:
  ```toml
  [[edge]]
  from = "SL-001"
  rel  = "implements"
  to   = "REQ-007"
  ```
  Validity is restricted by source/target family in Rust validation, not a global
  enum. **Edges that carry payload stay typed tables** — a spec's `members.toml`
  (`[[member]]`: requirement FK + label + order) and tech `interactions.toml`
  (`[[edge]]`: spec→spec, notes); future `coverage.toml` (status, artefact) — a
  generic edge would be lossy. The spec→requirement membership *is* the primary
  set, derived from `members.toml`, never stored twice (spec-entity-spec). The old
  `collaborators.toml` (cross-spec requirement reuse) is **dissolved** — that is
  the deferred `spec req link` verb, a second membership row.

## State vocabulary

- Lifecycle vocab is **family-specific**, not one global dialect. Keep the word
  **`status`** (slices already ship it; renaming built code to `state` is churn for
  no semantic gain — § Adjudication). Examples: slices
  `proposed|ready|started|audit|done` (slices-spec); coverage
  `planned|partial|verified|failed|blocked`; backlog `open|triaged|started|resolved|closed`.
- **Approval is not lifecycle.** If approval is modelled, it is a separate field
  (`approval = none|requested|approved|rejected`), never folded into `status`.

## Runtime state is not design schema

Hard boundary (already drawn in spec-entity-spec § Design-data vs runtime-state and
reservation-spec § Deferred): agent session state, handoff, review-index caches,
heartbeat/lease data, `phase.tracking` churn live under a separate
`.doctrine/state/` (mutable, likely gitignored or separately governed), **not** in
the same taxonomy as specs/slices/audits. Coordination is the reservation layer's
job.

## Rust implementation model

Three layers (consistent with B5's parse-vs-model split and relation-index's
registry):

```text
RawSpecToml    # tolerant parse; preserves unknowns (extra)
SpecEntity     # validated: typed ids, soft enums, normalized paths
SpecRegistry   # resolved references + FK diagnostics (relation-index § Two purposes)
```

Schemas are generated *from* Rust types, not hand-authored externally. Mutating
verbs write row + prose companions atomically and edit-preservingly (`toml_edit`),
never a full reserialize (spec-entity-spec / drift-spec § Known risks).

## Migration stance

Not a 1:1 port. An importer reads the spec-driver surface and writes the
normalized one (`frontmatter → entity.toml`, `spec.requirements → requirements.{toml,md}`,
`verification.coverage → coverage.toml`, `delta → slice`, `audit.findings →
findings.toml`, `workflow.* → state or dropped`). The live corpus
(`~/dev/spec-driver/.spec-driver/`) is the importer's input, later.

## What this does not change (sequencing guard)

This note is the **target taxonomy, not the next build.** The roadmap is
unchanged: slice-003 (engine + design-doc sibling) → IP + phases → spec family →
backlog/decision/audit. The essay's "minimal v1" (spec + backlog + decision +
audit at once) is more than the roadmap sequences; each entity still lands behind
its registry gate, one at a time, via the supersede pattern — not by this note
expanding scope.

## Adjudication

- **Accept as direction:** the storage rule, no-embedded-blocks, entity/facet
  consolidation, generic-edge-table-with-typed-exceptions, canonical+numeric id,
  family-specific vocab, approval-separate-from-lifecycle, runtime-state under
  `.doctrine/state/`, three-layer Rust model, importer-not-port. Most was already
  latent across the per-entity notes; this note names it.
- **Reject `status` → `state` rename:** pure churn against shipped slice code and
  template; the word carries no semantics the rename adds. Keep `status`.
- **Resolved — `revision` is a standalone change-axis kind (ADR-013):**
  spec-driver's `RE-` is a single change-record file, semantically on the *change*
  side (slice/audit), not a spec's identity. The home question is **closed**:
  `REV-NNN` is a first-class work-lifecycle kind peer to slice and REC — **not** a
  spec subtype and **not** a slice facet. Two forces decide it (ADR-013): the
  SL-060 gradient-inversion invariant (governance→work dependency needs a
  work-lifecycle anchor to route through, not the evergreen doc) and ADR-009's
  "approval is not lifecycle" (a Revision stages truth-writes and gates them on an
  orthogonal approval field — neither a status-less spec subtype nor a single
  slice's FSM can carry that). Mechanism: SL-066 `design.md`.
- **Open — per-subtype facet sets:** product vs tech carry different facet
  combinations (user-confirmed); the exact PRD/REV sets are pinned when those
  subtypes are designed (spec-entity-spec § Open questions 4).
