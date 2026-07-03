# Design SL-196: Per-edge relation descriptor

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Concept-map / ontology-like usage wants lightweight, human-readable nuance on a
relation edge — "frames estimation as attention burden", "contrasts with
story-point estimation" — without minting a new formal label or role for each
shade (which would bloat the closed `RelationLabel` vocabulary and defeat
ADR-016's structure/intent split). Descends from **IMP-244 part 2** (part 1, the
`concept`/CPT kind, is a separate off-backlog track).

The nuance is *free text with no graph meaning*: it must be stored, round-tripped,
rendered, and searchable, but inert to validation, inference, graph
traversal/effects, reciprocity, and edge identity.

## 2. Current State

- **`RelationLabel`** (`relation.rs:45`) is the closed outbound vocabulary; the
  **`RELATION_RULES`** table (`relation.rs:374`) keys `(source, label, role)` to
  five axes: `target`, `tier`, `link` policy, `inbound_name`, and — the load-bearing
  precedent — **`degree_bearing: bool`** (`relation.rs:363`), a per-label column
  gating an optional per-edge facet (`Degree`, SL-176) on exactly the `Fulfils` row.
- **`Degree`** is the working template for "optional, per-edge, identity-excluded,
  per-label-gated facet":
  - Rule column `degree_bearing`; gate in `validate_link` (`:1447`) — a degree on a
    non-bearing label bails (`DegreeNotApplicable`), symmetric to `RoleNotApplicable`.
  - Serialized on the Tier::One `[[relation]]` row only when present (`:1186`),
    load-bearing for diff stability.
  - **Excluded from edge identity**: identity is the `(label, role, target)` triple
    (`row_matches`, `:1265`); `append_relation_row` (`:1136`) matches the triple,
    then Noops on identical degree / hard-rejects a differing degree
    ("unlink to change").
  - Read struct `RelationEdge.degree` (`:746`); inbound render recovers it via
    `inbound_degree_index` (`relation_graph.rs:308`).
- **Storage tiers** (ADR-010): `Tier::One` = uniform `[[relation]]` rows;
  `Tier::Typed` = bespoke stores. Among the broad labels IMP-244 lists:

  | label | `inbound_name` | directed? | tier |
  |---|---|---|---|
  | `contextualizes` | `contextualized_by` | **directed** | One |
  | `references:concerns` | `concerned by` | **directed** | One |
  | `related` | `related` | symmetric | One |
  | `interactions` | `interactions` | symmetric | Typed |

- **`interactions`** (`spec.rs`) is bespoke: `{target, r#type, notes: Option<String>}`,
  authored via `spec interaction add` (**not** `link`), rendered at `spec.rs:1037`.
  Its `notes` is already free text — a symmetric-edge annotation. 8 rows live in
  `.doctrine/spec/tech/*/interactions.toml` (`type ∈ {feeds×3, uses×5}`).
- **Search** (`search.rs:124`) indexes `title + body` (the `.md`) **only**;
  relation TOML and `interactions.toml` are unindexed today.
- **Hydrated catalog** (`catalog/hydrate.rs`): `CatalogEdge` (`:129`) carries
  `label/role/target/origin` — **no facet payload** (degree bypasses it via
  re-scan). `entity_lex_doc` receives a `CatalogEntity`, whose edges live
  separately on `Catalog.edges`.

## 3. Forces & Constraints

- **ADR-004** — relations stored outbound-only; reciprocity derived. Descriptor
  rides the outbound row; no authored inbound descriptor.
- **ADR-010** — contract/write-seam unified, storage bespoke. Descriptor rides the
  Tier::One `[[relation]]` seam; the symmetric `interactions.notes` stays in its
  bespoke store.
- **ADR-016** — closed label + closed role. Descriptor adds **no** label or role;
  it is a third, non-semantic attribute (the "deciding principle" ruling: a new
  label is for structure/graph-effect/inbound change, a new role for intent —
  descriptor is neither).
- **Behaviour-preservation gate** — the entity engine is shared machinery; existing
  relation suites must stay green unchanged (absent descriptor ⟹ prior behaviour).
- **House lint** — Strings via `Vec<String>+concat`, no `push_str(&format!(…))`
  (`mem.pattern.lint.string-build-no-push-format`).

## 4. Guiding Principles

- **Ride the `Degree` seam; do not invent a parallel facet mechanism.** Descriptor
  is `Degree` re-skinned as free text — same rule column, gate, serialization,
  identity-exclusion, and conflict-reject shape.
- **Name the annotation by the edge's directedness.** `descriptor` refines a
  *directed* edge (A→B); `notes` annotate a *symmetric* pairing ({A,B}). The
  model's own `inbound_name` is the objective criterion:
  `inbound_name == label.name()` ⟺ symmetric. This slice builds `descriptor` for
  the directed broad labels and leaves the symmetric `interactions.notes`
  untouched.
- **Additive, not migratory.** No `notes` rename, no data migration, no template
  change.

## 5. Proposed Design

### 5.1 System Model

Descriptor is an **optional free-text cell on a Tier::One `[[relation]]` row**,
admissible on exactly the two *directed* broad labels: `contextualizes` and
`references` with `role = concerns`. It is inert to all graph semantics and
excluded from edge identity. It surfaces on **outbound** relation render and in the
**search index**. The symmetric labels (`related`, `interactions`) do **not** take
descriptor; `interactions` keeps its existing `notes`; `related` gets no free text
in this slice (YAGNI — would take `notes` if demand arises).

### 5.2 Interfaces & Contracts

**Rule table** — add a per-label column (mirrors `degree_bearing`):
```rust
// RelationRule (relation.rs:340)
pub(crate) descriptor_bearing: bool,   // true ONLY on contextualizes + references:concerns
```

**Write seam** — one param threaded beside `degree`:
```rust
validate_link(src_kind, label_str, role, degree, descriptor: Option<&str>) -> Result<&RelationRule>
append_edge(path, label, role, degree, descriptor: Option<&str>, target) -> AppendOutcome
append_relation_row(text, label, role, degree, descriptor: Option<&str>, target) -> (String, AppendOutcome)  // pure
run_link(source, label, target, role, degree, descriptor: Option<&str>, …)
resolve_link_path(root, source, label, role, degree, descriptor)  // forwards to validate_link
```

**Gate** — one clause after the degree gate (`validate_link`, `:1447`):
```rust
if descriptor.is_some() && !rule.descriptor_bearing {
    anyhow::bail!("`{label_str}` does not take a descriptor; remove `--descriptor`");
}
```

**CLI** — `--descriptor <TEXT>` on the `link` subcommand (`commands/cli.rs`).
Raw text (no parse-enum helper); validated **non-empty** (trim; empty/whitespace →
`"descriptor must be non-empty"`). The memory-source branch (`run_link`, `:89`)
**refuses** `--descriptor`, symmetric to its `--role` refusal (memory→entity
descriptors deferred).

**Read + hydrate**:
```rust
RelationEdge.descriptor: Option<String>   // relation.rs:735, beside .degree
CatalogEdge.descriptor: Option<String>    // catalog/hydrate.rs:129 — the consumer-neutral home
```

### 5.3 Data, State & Ownership

On-disk shape — new cell written **only when present** (load-bearing for diff
stability); facets cluster before `target`, `target` stays the trailing anchor:
```toml
[[relation]]
label = "contextualizes"
descriptor = "frames estimation as attention burden"
target = "SL-128"
```
```toml
[[relation]]
label = "references"
role = "concerns"
descriptor = "uses attention burden as the conceptual basis for bounded estimate fields"
target = "SL-128"
```

Ownership: descriptor is authored on the **source** entity's outbound row (ADR-004),
mutated only through `link` / `unlink`. `descriptor_bearing` (and its disjointness
from `degree_bearing`) is owned by `RELATION_RULES`.

### 5.4 Lifecycle, Operations & Dynamics

- **Add** — `link SL-x contextualizes SL-y --descriptor "…"` → gate → append row
  with the cell.
- **Re-link, same triple, identical descriptor** → `Noop` (no rewrite/mtime churn).
- **Re-link, same triple, differing descriptor** → **hard reject**
  (`"already contextualizes SL-y with descriptor=…; unlink to change"`). `unlink`
  + relink is the explicit change path.
- **Unlink** — matches `(label, role, target)`; descriptor ignored (`:371`).
- **Facet-conflict is generalized** — `append_relation_row`'s same-triple branch
  compares *both* facets. Since `descriptor_bearing ∩ degree_bearing = ∅`, at most
  one is ever `Some`; Noop iff both equal, else bail naming the differing facet.
- **Render** — descriptor prints on the **outbound** edge in `inspect`/`show`
  (`contextualizes SL-128 — "…"`). **No** inbound render, **no**
  `inbound_descriptor_index` (diverge from degree — descriptor is verbose
  source-authored prose; duplicating on every target's inbound list bloats the view).
- **Search** — `hydrate` populates `CatalogEdge.descriptor`; the search doc-build
  (`search.rs:336`) joins `Catalog.edges` by source and appends each entity's
  outbound descriptors to its lex text.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** — descriptor is excluded from edge identity: `row_matches` /
  `relation_row_find` key on `(label, role, target)` only.
- **INV-2** — `descriptor_bearing` is true on exactly `{contextualizes,
  references:concerns}` and disjoint from `degree_bearing` (no edge carries both).
- **INV-3** — a `[[relation]]` row serializes `descriptor` iff non-empty-present;
  absent otherwise (diff stability).
- **INV-4** — existing relation behaviour is unchanged when no descriptor is
  authored (behaviour-preservation).
- Edge cases: empty/whitespace descriptor rejected; multi-line / special chars
  round-trip via `toml_edit::value` escaping; descriptor on a non-bearing or
  memory-source edge rejected; descriptor never affects reciprocity, priority,
  traversal, or dedup.

## 6. Open Questions & Unknowns

- **OQ-1 (resolved)** — reject vs ignore on illegal placement → **reject**, the
  established house policy (degree/role gate).
- **OQ-2 (resolved)** — append-conflict on differing descriptor → **reject**
  (degree precedent).
- **OQ-3 (resolved)** — `interactions` treatment → keep `notes` (symmetric),
  descriptor is directed-only.
- **OQ-4 (deferred)** — memory-source descriptors → refused this slice.

## 7. Decisions, Rationale & Alternatives

- **D1 — descriptor rides the `Degree` seam.** Alternative: a bespoke free-text
  mechanism. Rejected — parallel implementation; `Degree` already proves the exact
  shape (optional, identity-excluded, per-label-gated).
- **D2 — split annotation by directedness (`descriptor` vs `notes`), not unify.**
  Alternative (considered, then rejected): one `descriptor` everywhere, migrating
  `interactions.notes`. Rejected — flattens a real, model-encoded distinction
  (`inbound_name`), forces a breaking `--notes` rename + 8-row migration, and puts
  a "descriptor" on symmetric edges where direction is meaningless. The split is
  *less* work and *more* honest.
- **D3 — outbound-only render (diverge from degree's inbound index).** Rationale:
  verbose prose vs terse marker.
- **D4 — descriptor on the hydrated `CatalogEdge`** (not degree's re-scan
  side-path). Rationale: consumer-neutral home; search + any future consumer
  project from one structure (`Catalog` design intent, `hydrate.rs:62`).
- **D5 — `related` gets no free text now.** Symmetric; would take `notes`; no
  demand (YAGNI). Deferred.

## 8. Risks & Mitigations

- **R1 — signature churn** across `validate_link`/`append_edge`/`run_link` +
  constructor sites. *Mitigation*: compile-driven; `descriptor: None` default
  mirrors the `degree: None` defaults already at `:768/780`.
- **R2 — search doc-build coupling** (join `Catalog.edges` by source).
  *Mitigation*: `CatalogEdge.descriptor` keeps it a pure projection; no re-parse.
- **R3 — behaviour-preservation regressions** in shared relation machinery.
  *Mitigation*: existing suites run green unchanged; new VTs are additive.
- **R4 — verbose/multiline descriptor render noise**. *Mitigation*: outbound-only;
  formatting (truncate/wrap) is a phase-plan concern.

## 9. Quality Engineering & Validation

VT set (mirrors the `Degree` VTs at `relation.rs:3106+`):

| VT | Behaviour |
|---|---|
| VT-1 | Round-trip: bearing-label row with `descriptor="…"` writes and reads back |
| VT-2 | Identity-excluded: same triple + identical descriptor → Noop; differing → reject |
| VT-3 | Gate: `--descriptor` on a non-bearing label (`supersedes`/`related`/`interactions`/`fulfils`) → reject |
| VT-4 | Accept: the 2 bearing labels accept `--descriptor` |
| VT-5 | Empty/whitespace descriptor rejected |
| VT-6 | Unlink triple-match: descriptor ignored |
| VT-7 | On-disk order `label / role? / descriptor? / target` |
| VT-8 | Render: descriptor on outbound `inspect`; absent on inbound |
| VT-9 | Search: entity with a descriptor edge found by descriptor text; `CatalogEdge.descriptor` populated |
| VT-10 | Disjointness pin: `descriptor_bearing ∩ degree_bearing = ∅`; true exactly on `{contextualizes, references:concerns}` |

Behaviour-preservation (VA/VH): existing relation + catalog suites green unchanged.

## 10. Review Notes

(internal adversarial pass pending; then external GPT inquisition per handoff.)
