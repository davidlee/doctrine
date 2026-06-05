# Design SL-015: Spec entity v1: product + technical specs

## 1. Design Problem

Land first-class **product** and **technical** specs on the existing entity engine
([`src/entity.rs`](../../../src/entity.rs)), modelled on Spec-Driver but reframed
to the Doctrine way: **a spec is a thin aggregate root over freestanding
`requirement` entities joined by membership edges** — not a monolith that owns
embedded requirement rows. The pre-design reframe (slice-015.md § Design
Direction, D1–D7) is the input; this design formalises it and rewrites the two
note sections it overturns.

The hard parts are not the scaffold (the engine already does that):

1. **Requirement as a peer entity** with a two-tier identity — a durable reserved
   id plus a sticky, human-friendly local label that lives on the membership edge.
2. **The FK-validation pass** (`spec validate`) co-landing, because tech specs
   introduce the first cross-entity references.
3. **Two-tree atomicity** of `spec req add` (reserve a requirement in one tree,
   append a membership row in another).
4. **Render** as the one-way readable whole — the only way to read a spec as a
   document, and a strict derived view (no bidirectional sync, ever).

Discipline (CLAUDE.md, SL-006 precedent): mirror `adr.rs`/`slice.rs` over
`entity.rs` **unchanged** (behaviour-preservation gate); no fork, no parallel
implementation; use the CLI shapes that already exist.

## 2. Current State

- [`src/entity.rs`](../../../src/entity.rs) is a kind-agnostic scaffolding engine
  serving two identity shapes — numeric (`MaterialiseRequest::Fresh`, `mkdir`
  reservation, `max+1`, `{id:03}`, canonical `<PREFIX>-NNN`) and named (seam A).
  It materialises *filesets*, explicitly **not** row-appends / table mutations
  (`refuse_clobber` comment). Proven by `slice`, `adr`, `memory` callers.
- [`src/adr.rs`](../../../src/adr.rs) is the mirror precedent: a `Kind` const
  (`dir`/`prefix`/`scaffold`), `render_*_toml`/`render_*_md`, `materialise(Fresh)`,
  `list_rows`, and `set_adr_status` — a `toml_edit` in-place mutation of a
  **committed** file with **no progress log** (git history is the trail).
- Specs exist only as `doc/*` prose intent + placeholder `spec-product`/`spec-tech`
  skills ("Not yet structural"). No registry / FK validation exists.
- Glossary reserves `PRD-001` / `SPEC-001` / `REV-001`.

## 3. Forces & Constraints

- **Storage rule** (entity-model.md): identity + typed references in TOML; prose
  in MD; **never queried/derived data in committed prose**. Facets are flat
  arrays-of-tables. Templates are write-once defaults — tooling never parses prose
  structure.
- **Behaviour-preservation gate**: `entity.rs` and the slice/adr/memory suites stay
  green **unchanged**.
- **Pure/imperative split**: id/slug/fileset decided from inputs; only `claim` and
  writes touch disk.
- **Integrity co-lands** (spec-entity-spec § Known risks): the validation that
  catches dangling FKs ships in the same slice as the tables that introduce them.
- **Edge rule** (entity-model.md): payload-free links → generic edge table;
  **payload-bearing edges → typed tables.** Both v1 edges carry payload.
- **No bidirectional sync, ever** (user, locked): render is one-way derived.

## 4. Guiding Principles

- Requirements are the durable atoms; specs and features are aggregations over
  them. **Identity immutable, membership mobile.**
- Reuse the engine and the ADR caller shape; generalise only as far as a second
  identity shape forces (engine memory `mem.system.engine.identity-claim-seam`).
- Queryable things stay structured (TOML facets); narrative stays prose.
- Derive, don't store (membership *is* the `primary` set; reverse refs are
  registry traversals).
- One-way derivation only: sources → render. Editing is component-level.

## 5. Proposed Design

### 5.1 System Model

Entity set for v1: **`requirement`** (peer) and **`spec`** with subtypes
**`product`** / **`tech`** (aggregate roots). `feature` is designed (§6/§7) but
**not built** — forward-compatible because it is all edges over reserved
requirements.

```
.doctrine/
  requirement/NNN/                 REQ-NNN  (one global tree, one reservation ns)
    requirement-NNN.toml           kind, status, acceptance_criteria[]
    requirement-NNN.md             statement, rationale (prose)
  spec/product/NNN/                PRD-NNN  (own tree + reservation ns)
    spec-NNN.toml                  light identity (no tech flat fields)
    spec-NNN.md                    prose: Problem · Value · Principles · Outcomes · Out-of-scope
    members.toml                   [[member]] requirement, label, order
  spec/tech/NNN/                   SPEC-NNN (own tree + reservation ns)
    spec-NNN.toml                  identity + category, c4_level, sources[], responsibilities[]
    spec-NNN.md                    prose (incl. concerns/hypotheses/decisions sections)
    members.toml                   [[member]] requirement, label, order
    interactions.toml              [[edge]] target, type, notes   (tech only)
```

What the note's seven tech blocks became (the narrowed decomposition, locked Q2):
`requirements` → peer entity; `capabilities` → the deferred `feature` entity;
`relationships.primary` → derived from membership; `relationships.collaborators`
→ **dissolved** (no owner ⇒ no "mine vs yours", just membership);
`relationships.interactions` → `interactions.toml`; `coverage` → **designed,
deferred** (needs verification artefacts that do not exist in v1);
`concerns`/`hypotheses`/`decisions` → **prose sections** in `spec-NNN.md`.

### 5.2 Interfaces & Contracts

CLI (`spec` subcommand tree; `requirement` has no standalone surface in v1 —
spec-mediated):

- `spec new <product|tech> "<title>" [--slug S]` — `materialise(Fresh)` on the
  subtype `Kind`. Prints `PRD-NNN` / `SPEC-NNN`.
- `spec req add <spec-ref> "<title>" --kind <functional|quality> [--label FR-001]`
  — reserve a requirement, assign a label, append the membership row (§5.4).
- `spec show <spec-ref>` — render the readable whole to **stdout** (§5.4). v1 is
  ephemeral; `--write` (materialised `*.rendered.md`) is deferred (§7 D9).
- `spec validate [--path-scope …]` — the FK-validation pass (§5.4). Exit non-zero
  on any hard finding.
- `spec list [--status S]` — rows per subtype: id, status, slug, `#members`. Mirror
  of `adr list` / `slice list`.

`<spec-ref>` accepts `PRD-3` / `SPEC-12` (canonical) or the numeric id within a
subtype context. `spec req add … --label`-less auto-assigns the next `FR-`/`NF-`
for the kind within that spec.

### 5.3 Data, State & Ownership

Parse-layer types (entity-model three-layer split — tolerant parse → validated →
registry; only the parse layer is pinned here):

```rust
// requirement.rs
enum ReqKind   { Functional, Quality }                       // closed; kebab serde
enum ReqStatus { Pending, Active, Deprecated, Superseded }   // closed
struct Requirement {
    id: u32, slug: String, status: ReqStatus, kind: ReqKind,
    #[serde(default)] acceptance_criteria: Vec<String>,      // testable list — stays structured
}

// spec.rs
enum SpecSubtype { Product, Tech }                           // closed; selects tree/prefix/fileset
enum SpecStatus  { Draft, Active, Deprecated, Superseded }   // closed
struct Spec {
    id: u32, slug: String, name: String, status: SpecStatus, kind: SpecSubtype,
    // tech-only flat fields; absent/default for product:
    #[serde(default)] category: Option<String>,
    #[serde(default)] c4_level: Option<String>,
    #[serde(default)] responsibilities: Vec<String>,
    #[serde(default, rename = "source")] sources: Vec<Source>,
}
struct Member      { requirement: String, label: String, order: u32 }   // FK open
struct Interaction { target: String, #[serde(rename = "type")] kind: String,
                     #[serde(default)] notes: Option<String> }           // FK open
```

Ownership: a spec **owns** its `members.toml` and `interactions.toml` (forward
edges, source-side). A requirement is **passive** — it owns only its intrinsic
facts (kind, status, criteria, statement); it does not know who members it
(reverse = registry traversal). The sticky label is **membership state**, not
requirement state — so the same `REQ-NNN` carries different labels under different
specs (the many-to-many the reframe bought).

### 5.4 Lifecycle, Operations & Dynamics

**`spec new`** — one `materialise(Fresh)` on `PRODUCT_SPEC_KIND` or
`TECH_SPEC_KIND`; the subtype's `scaffold` fn returns its fileset (product: 3
files; tech: 4). Pure mirror of `adr run_new`.

**`spec req add`** (the two-tree write):
1. Resolve the spec dir (err if absent).
2. `materialise(Fresh)` `REQUIREMENT_KIND` → reserve `REQ-NNN`, scaffold its two
   files. (Engine claim = atomic; reservation makes id-collision impossible.)
3. Assign the label: explicit `--label`, else auto = next free `FR-NNN`
   (functional) / `NF-NNN` (quality) scanning the spec's existing `members.toml`.
4. **Edit-preserving** append (`toml_edit`, not serde reserialize — preserves
   comments / unknown keys) of `[[member]] requirement="REQ-NNN" label="…"
   order=<max+1>` to the spec's `members.toml`.

Atomicity: steps 2 and 4 cross two trees and are **not** transactional. The only
failure window (reserve succeeds, append fails) yields an **orphan requirement** —
reserved, un-membered, inert and harmless; `validate` warns on it. Accepted
(slice-015.md). Engine H2 still guarantees step 2 itself leaves no partial dir.

**`spec show` / render** — pure reassembly over parsed facets (no mutation, no
write): spec identity + flat fields → prose body verbatim → a **Requirements**
section (per member in `order`: `### FR-001 (REQ-007) — <title>`, then kind,
statement, acceptance criteria) → interactions → registry-resolved inbound refs.
**v1 is stdout-only and ephemeral**, so it is a pure function of present state and
**cannot go stale** (§7 D9).

**`spec validate`** — lazy, command-scoped load (relation-index § lazy loading):
scan the three trees into id sets + an edge list, then check:

| check | severity |
|---|---|
| every `members[].requirement` resolves to a requirement id | **hard** (dangling FK) |
| every `interactions[].target` resolves to a spec id | **hard** (dangling FK) |
| `label` unique within a spec's members | **hard** (duplicate) |
| duplicate requirement id / spec id across a merged tree | **hard** |
| requirement membered by ≥1 spec | **warn** (orphan) |

Cache-independent (no index persisted — the relation-index *cache* is deferred).
**Cycle detection** arrives with the feature DAG (deferred), so v1 validates
existence/uniqueness only.

**Status** — `spec` and `requirement` `status` are **hand-edited** in v1 (ungated,
git is the trail), consistent with slices today. No `spec status` verb (the
`adr status` `toml_edit` mirror is a trivial later add). Requirement lifecycle is
advanced by the *change process* when it exists (deferred); v1 source of truth is
`requirement-NNN.toml`.

### 5.5 Invariants, Assumptions & Edge Cases

- **Identity immutable, membership mobile.** `REQ-NNN` and a once-assigned label
  never change; relocation re-points membership rows.
- **Labels never renumbered.** Auto-assign fills the next free `FR-`/`NF-`; a
  retired label is never reused. (Immutability is a *process* rule — not
  enforceable at rest; mitigated by auto-assign + the uniqueness lint.)
- **Forward-only edges**, stored source-side; no backlinks (reverse via registry).
- **`entity.rs` unchanged** — the gate.
- Edge cases: empty `members.toml` (valid — a spec may have no requirements yet);
  product spec has no `interactions.toml` (absent, not empty); a requirement
  membered by both a product and a tech spec (valid — distinct labels); merge
  introduces a duplicate label (caught hard at validate over the merged file).

## 6. Open Questions & Unknowns

1. **Requirement durable-id width** — `{id:03}` (`REQ-007`) follows the engine; no
   reason to diverge. *Leaning: keep engine format.*
2. **Auto-label collision across merges** — two branches each auto-assign `FR-009`.
   Detection-only (validate hard lint over the merged file), same shape as the
   id-collision lint. No prevention attempted in v1.
3. **`feature` facet set & DAG mechanics** — designed at the level needed to prove
   forward-compatibility (membership + dependency edges over requirements); the
   full sheet is pinned when the feature slice is scoped.
4. **Requirement lifecycle source-of-truth** once the change process exists —
   `requirement-NNN.toml` vs derived from completed changes. Deferred with that
   process.

## 7. Decisions, Rationale & Alternatives

Reframe decisions D1–D7 are recorded in slice-015.md § Design Direction. This
design adds:

- **D-Q1 — requirement is a reserved numeric peer entity** (not a facet row, not a
  global table). *Why:* rides `entity.rs` `Fresh` unchanged; reservation turns the
  note's merge-collision risk into an impossibility; label-on-edge gives the
  two-tier id. *Rejected:* global `requirements.toml` (hand-rolled reservation +
  shared mutable file = merge churn); per-spec facet row (single-owner, the thing
  the reframe overturned).
- **D-Q2 — full decomposition collapse.** Only `members` + `interactions` are
  structured; coverage deferred; concerns/hypotheses/decisions → prose. *Why:*
  loses no queryable power (criteria on the requirement entity; coverage
  deferred-not-dropped; edges validated). *Rejected:* keeping the note's seven
  facet pairs (the ceremony the slice exists to trim).
- **D-Q3 — per-spec typed facet files; no generic edge table in v1.** Both edges
  carry payload ⇒ typed (entity-model rule). Generic table deferred until the
  payload-free feature DAG forces its shape.
- **D-Q4 — pragmatic v1 boundary.** interactions hand-authored (no verb), status
  hand-edited, requirement CLI spec-mediated. *Why:* keeps the surface minimal;
  each deferred verb is an additive mirror.
- **D-Q5 — product reuses the requirement entity + membership verbatim**; differs
  from tech only by the absent `interactions.toml` + tech flat fields. *Rejected:*
  a separate `product_requirements` model (parallel implementation).
- **D8 — no bidirectional sync, ever** (architectural invariant). Render is
  one-way derived; editing is component-level; the rendered doc is never an input.
  This permanently excludes spec-driver's `sync` adapters and the
  edit-the-document-back-propagate problem.
- **D9 — render is ephemeral in v1; materialised render is derived-tier and
  deferred.** `spec show` → stdout, a pure function of present state ⇒ no staleness
  exists. A committed `*.rendered.md` (for git-browsable reading / a CI
  point-in-time dump) is **derived tier** (gitignored, regenerable, never trusted,
  never edited); if ever published, its freshness rides the **already-shipped boot
  snapshot sentry pattern** (SL-011: regenerate + `--check` content-hash sentry).
  The render's inputs are the spec's facets *plus* its membered requirement
  entities, so a requirement edit fan-out-invalidates dependent renders — the
  *efficient incremental* form of that invalidation **is** the deferred
  relation-index cache, so it introduces no new hard problem. Future read/edit UX
  is a web sidecar through the CLI + a CI dump-to-disk; the only residual
  consistency concern is document-search indexing.

## 8. Risks & Mitigations

- **Entity-count at scale** — thousands of requirement dirs in a large product.
  *Mitigation:* the standard entity cost; render recovers read-locality; the
  relation-index budget ("thousands of edges/files, single-digit MB") covers it.
- **Render correctness** — load-bearing as the only readable whole. *Mitigation:*
  pure function, directly tested; one-way (D8) keeps it trivially correct.
- **Two-tree orphan requirement** — benign + validate warn (§5.4).
- **Label immutability is a process rule** — not enforceable at rest.
  *Mitigation:* auto-assign + uniqueness/duplicate hard lints; never renumber.
- **Behaviour-preservation** — extending nothing in `entity.rs`; new callers only.
  *Mitigation:* the existing suites are the proof; they stay green unchanged.

## 9. Quality Engineering & Validation

TDD red/green/refactor. Behaviour gate: `entity.rs` + slice/adr/memory suites green
**unchanged**. New tests (titles):

- `requirement_scaffold_lays_out_toml_md` · `product_spec_scaffold_is_light_3_files`
  · `tech_spec_scaffold_has_members_and_interactions`.
- `spec_req_add_reserves_requirement_and_appends_member` ·
  `spec_req_add_is_edit_preserving` (comments/unknown keys survive) ·
  `spec_req_add_auto_labels_fr_then_nf_by_kind` ·
  `spec_req_add_orphan_on_append_failure_is_inert`.
- `validate_flags_dangling_member_fk` · `validate_flags_dangling_interaction_target`
  · `validate_flags_duplicate_label_in_spec` · `validate_warns_orphan_requirement`
  · `validate_passes_clean_corpus`.
- `render_reassembles_members_in_order` · `render_is_pure_no_write`.
- `spec_list_rows_per_subtype_with_member_count`.

Gate: `cargo clippy` zero warnings (bins/lib, not `--all-targets`); `just check`.

## 10. Review Notes

_(Adversarial pass pending — to be recorded here.)_
