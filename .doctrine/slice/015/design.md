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
- `spec show <spec-ref>` — render the readable whole to **stdout** (§5.4): own
  content + members + outbound interactions. **Inbound refs deferred** (R3 — they
  force a full-corpus scan + the unpersisted registry; they belong to the registry
  surface, not local `show`). v1 ephemeral; `--write` (`*.rendered.md`) deferred
  (§7 D9).
- `spec validate [<spec-ref>]` — the FK-validation pass (§5.4); whole-corpus by
  default, one spec's outbound FKs if given. Exit non-zero on any hard finding.
- `spec list [--status S]` — rows per subtype: id, status, slug, `#members`. Mirror
  of `adr list` / `slice list`.

`<spec-ref>` **requires the canonical prefix** (`PRD-NNN` / `SPEC-NNN`) on `req
add` / `show` / `validate` — those verbs carry no subtype selector, so a bare
numeric is ambiguous across the two independent reservation namespaces
(`spec/product/NNN` and `spec/tech/NNN` each start at 1). `spec new` is the only
verb that names the subtype. `spec req add … --label`-less auto-assigns the next
`FR-`/`NF-` for the kind within that spec.

### 5.3 Data, State & Ownership

Parse-layer types (entity-model three-layer split — tolerant parse → validated →
registry; only the parse layer is pinned here):

```rust
// requirement.rs
enum ReqKind   { Functional, Quality }                       // closed; kebab serde
enum ReqStatus { Pending, Active, Deprecated, Superseded }   // closed
struct Requirement {
    id: u32, title: String, slug: String,                    // title (shared-Meta convention); slug derived from it
    status: ReqStatus, kind: ReqKind,
    #[serde(default)] description: Option<String>,           // optional one-line summary (queryable);
                                                             //   full statement/rationale stays prose in .md
    #[serde(default)] tags: Vec<String>,                     // uniform tag seam (see below)
    #[serde(default)] acceptance_criteria: Vec<String>,      // testable list — stays structured
}

// spec.rs
enum SpecSubtype { Product, Tech }                           // closed; selects tree/prefix/fileset
enum SpecStatus  { Draft, Active, Deprecated, Superseded }   // closed
enum C4Level     { Context, Container, Component, Code }      // closed; kebab serde; tech-only
struct Spec {
    id: u32, slug: String, title: String, status: SpecStatus, kind: SpecSubtype,
    #[serde(default)] tags: Vec<String>,                     // uniform tag seam (see below)
    // tech-only flat fields; absent/default for product:
    #[serde(default)] category: Option<String>,              // deliberately OPEN vocab (domain taxonomy, drifts by design)
    #[serde(default)] c4_level: Option<C4Level>,             // closed C4 set (C6 ruling)
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

**Tags — a uniform cross-entity seam (D10).** Every authored entity (requirement,
spec, and `feature` when it lands) carries an optional `tags: Vec<String>`
(`#[serde(default)]`, on the identity TOML). v1 **parses and round-trips** the
field edit-preservingly — nothing more. Tag *semantics* (a tag index, `--tag`
query/filter, a controlled vocabulary, tag-scoped views) are **designed-deferred**:
deliberately one design, applied uniformly, not reinvented per entity. There is a
latent unification with memory's existing tag scoping (`doctrine memory find
--tag`, `scope.tags`) — the eventual tag query should reuse that vocabulary rather
than fork it (§6 Q5). The seam is added now (while "design is free") so the field
exists from day one and a later tag feature needs no migration of authored files.

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
   order=<max+1>` to the spec's `members.toml`. The file is **scaffold-seeded
   empty** by `spec new` (precondition, §5.1), so the append always has a file to
   open.

Atomicity: steps 2 and 4 cross two trees and are **not** transactional. The only
failure window (reserve succeeds, append fails) yields an **orphan requirement**.
Since every requirement is born membered, an orphan is **evidence of a torn write**,
not benign drift — so `validate` flags it **hard** (exit non-zero), not warn (C5
ruling). The reserved dir is left uncommitted; cleanup is the operator's
(`git`-clean / `rm`). Engine H2 still guarantees step 2 itself leaves no partial
dir.

**`spec show` / render** — pure, **local** reassembly over parsed facets (no
mutation, no write, no cross-corpus scan): spec identity + flat fields → prose body
verbatim → a **Requirements** section (per member in `order`: `### FR-001 (REQ-007)
— <name>`, then kind, statement, acceptance criteria — each member's requirement
read by canonical FK `REQ-NNN` → `requirement/NNN/`) → outbound interactions.
Inbound refs are **not** shown (R3 — a registry query, deferred). `order` is
advisory (stable-sort key); gaps/dups are cosmetic, not validated. **v1 is
stdout-only and ephemeral**, so it is a pure function of present *local* state and
**cannot go stale** (§7 D9).

**`spec validate`** — lazy, command-scoped load (relation-index § lazy loading):
scan the three trees into id sets + an edge list, then check:

| check | severity |
|---|---|
| every `members[].requirement` (canonical `REQ-NNN`) resolves to a requirement | **hard** (dangling FK) |
| every `interactions[].target` resolves to a spec id | **hard** (dangling FK) |
| `label` unique within a spec's members | **hard** (duplicate) |
| requirement membered by ≥1 spec | **hard** (orphan = torn write) |

FK strings are stored **canonical** (`REQ-007`, `SPEC-012`) and parsed to the
numeric dir on resolve. No **id-collision** check: like slice/adr, `mkdir`
reservation + git add/add conflict handle duplicate *entity* ids before any lint
(R2) — the only silent merge risk is a duplicate **label row**, which the
uniqueness check above covers. Label assignment + `order` are racy under
concurrent `req add` to one spec (TOCTOU); the uniqueness lint is the backstop,
consistent with the accepted two-tree non-atomicity. Cache-independent (the
relation-index *cache* is deferred); **cycle detection** arrives with the feature
DAG (deferred) — v1 validates existence/uniqueness only.

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

### 5.6 Code Impact

- `src/requirement.rs` *(new)* — `REQUIREMENT_KIND` (dir `requirement`, prefix
  `REQ`), scaffold, `render_*_toml`/`_md`. Mirror of `adr.rs`.
- `src/spec.rs` *(new)* — `PRODUCT_SPEC_KIND` (dir `spec/product`, prefix `PRD`) +
  `TECH_SPEC_KIND` (dir `spec/tech`, prefix `SPEC`); commands `new` / `req add` /
  `show` / `validate` / `list`; the `members.toml` edit-preserving append.
- `src/registry.rs` *(new — relation-index seed)* — the cache-independent
  FK-validation pass; kept minimal (generalise only as far as forced).
- `src/entity.rs` — **unchanged**; gains three `Kind`/`Fresh` callers only (R6 gate).
- `src/meta.rs` — spec/requirement `Meta` for `list`, **additive only** — the
  shared slice/adr `Meta` path must not change (R6: behaviour gate's sharp edge).
  The identity toml carries **`title`** (not `name`) so `read_metas` →
  `toml::from_str::<Meta>` parses it (C2 — `Meta` requires `title`, no default).
  `spec list`'s `#members` column rides the generic `meta::render_table` (the
  SL-009 slice-rollup path), **not** the fixed 4-column `meta::format_list` —
  genuinely additive, no shared mutation.
- `src/main.rs` / cli — `spec` subcommand tree wiring.
- `install/templates` — product / tech / requirement prose scaffolds.
- **`doc/` consistency sweep (R5, widened by inquisition C1/C3)** — the overturned
  compound-key + facet-row creed spans **four** files; a partial sweep leaves the
  canon self-contradictory (the gravest `/canon` sin):
  - `spec-entity-spec.md` — rewrite the **full** model, not two sections: §§ The
    decomposition + Mapping, Requirement identity, Spec identity, Metadata & table
    schemas (FK examples `:233`/`:251`), Lifecycle (supersede — `SPEC-110.FR-001`
    "resolvable forever", contradicted by D3 identity-immutable/membership-mobile),
    Follow-ups (the `show <SPEC-110.FR-001>` render example) (C3).
  - `entity-model.md` — the umbrella taxonomy, omitted by the original sweep (C1):
    § Entity-vs-facet ("rows, not artefacts", `:70` → now a peer **artefact**); §
    Identity and references (compound key `:82`/`:89` → durable `REQ-NNN` FK with
    `FR-`/`NF-` as membership labels); § Edges (`collaborators.toml`, `:93` →
    **dissolved** by the decomposition).
  - `relation-index.md` — **not** a compound-key strand (grep confirms none); the
    real taint is the facet-row taxonomy at `:52` ("~8 sister files … requirements/
    capabilities/coverage/… tables") — correct that, not a phantom FK repair.
  - `glossary.md` — holds only `PRD-001`/`SPEC-001`; **additively** add `REQ-` (the
    durable requirement id) + `FR-`/`NF-` (membership labels) rows — no repair.
  *Verify:* `grep -rE 'rows, not artefacts|SPEC-[0-9]+\.(FR|NF)' doc/` returns
  nothing the sweep does not name. A sweep that strands any of these is incomplete
  by §5.6's own standard.
- `.claude/skills/spec-product`, `spec-tech` — drop the "not yet structural" caveat.

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
5. **Tag semantics (D10)** — index, `--tag` query/filter, controlled vocabulary,
   tag-scoped views; and whether they unify with memory's `--tag`/`scope.tags`
   surface (they should). Field-only seam in v1; the feature is designed when first
   needed, once, across all entities.

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
  `spec_req_add_orphan_on_append_failure_left_uncommitted`.
- `validate_flags_dangling_member_fk` · `validate_flags_dangling_interaction_target`
  · `validate_flags_duplicate_label_in_spec` · `validate_flags_orphan_requirement_hard`
  · `validate_passes_clean_corpus`.
- `render_reassembles_members_in_order` · `render_is_pure_no_write`.
- `spec_list_rows_per_subtype_with_member_count`.
- `tags_and_description_round_trip_on_requirement_and_spec` (seam: parsed +
  preserved, no semantics).
- `spec_list_meta_parses_scaffolded_spec_toml` (C2: `meta::read_metas` reads the
  `title`-keyed identity toml the scaffold writes — the shared `Meta` round-trips).

Gate: `cargo clippy` zero warnings (bins/lib, not `--all-targets`); `just check`.

## 10. Review Notes

### Internal adversarial pass (integrated)

- **R1 — requirement lacked a structured `name`.** `req add "<title>"` + render's
  heading imply a stored title; struct only had `slug`. **Fixed** (§5.3 adds
  `name`; slug derives from it).
- **R2 — id-collision check was over-built.** `mkdir`-reserved ids git-conflict
  (add/add) before any lint, exactly as slice/adr already trust; only a duplicate
  *label row* is a silent merge risk. **Fixed** — dropped the id-dup check, kept
  label-uniqueness (§5.4).
- **R3 — inbound refs in `show` smuggled a full-corpus scan + unpersisted
  registry.** **Fixed** — deferred to the registry surface; v1 `show` is local
  (own content + members + outbound interactions), which also hardens D9's purity
  claim (§5.2/§5.4).
- **R4 — `spec validate --path-scope` was cargo-culted** from memory's flags.
  **Fixed** — whole-corpus by default, optional `[<spec-ref>]` (§5.2).
- **R5 — rewriting two `spec-entity-spec.md` sections strands compound-key refs**
  in `relation-index.md` / `glossary.md`. **Fixed** — §5.6 makes it a doc/
  consistency sweep, not a two-section edit.
- **R6 — `meta.rs` is shared.** **Fixed** — §4/§5.6 require the spec/requirement
  `Meta` parse to be additive; the slice/adr path must not change (the behaviour
  gate's sharp edge).
- **R7 — the design doc had no explicit Code Impact section** (scaffold lacked the
  slot). **Fixed** — added §5.6.

Residual (accepted, carried to §6/§8, not blocking): label/order TOCTOU under
concurrent `req add` (uniqueness lint is the backstop); auto-label cross-merge
collision (detection-only).

### External challenge — `/inquisition` pass (integrated)

The formal hostile pass (`inquisition.md`) **acquitted the load-bearing thesis** —
requirement-as-peer-entity riding `entity.rs` unchanged (§II gate held against the
rack) — and raised six seam-level charges. All dispositioned and integrated above:

- **C1 (grave) — incomplete + misdirected canon sweep. Accepted.** §5.6 widened to
  a **four-file** sweep (adds `entity-model.md` `:70`/`:82`/`:89`/`:93`); the false
  witness corrected — `relation-index.md` has no compound-key (facet-row at `:52`),
  `glossary.md` is additive (`REQ-`/`FR-`/`NF-` rows), not a repair.
- **C2 (grave) — `Meta` requires `title`, structs stored `name`. Accepted.** §5.3
  adopts `title` (the adr/slice convention; `name` was a gratuitous neologism that
  broke `read_metas`). §5.6 records `#members` rides `render_table`, not the fixed
  `format_list`; §9 adds `spec_list_meta_parses_scaffolded_spec_toml`.
- **C3 (serious) — primary target under-scoped. Accepted.** §5.6's
  `spec-entity-spec.md` clause widened from two sections to the full compound-key/
  facet model (decomposition, identity, schemas, supersede, render).
- **C4 (serious) — ambiguous bare-numeric `<spec-ref>`. Accepted.** §5.2 now
  **requires** the canonical `PRD-`/`SPEC-` prefix on `req add`/`show`/`validate`
  (no subtype selector on those verbs); bare-numeric struck.
- **C5 (moderate) — orphan severity. Accepted, hardened.** An orphan = evidence of
  a torn write (every requirement is born membered) ⇒ `validate` flags it **hard**,
  not warn (§5.4 + table); reserved dir uncommitted, operator-cleaned.
- **C6 (minor) — soft vocabulary. Accepted, split.** `c4_level` → closed `C4Level`
  enum (context/container/component/code); `category` stays deliberately-open
  `Option<String>` (domain taxonomy) (§5.3).
- **Q4 — Spec-Driver `decisions` were narrative-only** (never id'd / cross-
  referenced), so D-Q2's prose collapse loses no query — the trim is clean.

Verdict: **venial seam heresies, not mortal** — scope/reconciliation corrections,
no redesign. The thesis stands; the consistency surface is now closed.
