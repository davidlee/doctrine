# Design SL-196: Per-edge relation descriptor

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

We want a relation edge to carry lightweight, human-readable nuance — a free-text
statement *about* the relationship — without minting a new formal label or role
for each shade (which would bloat the closed `RelationLabel` vocabulary and defeat
ADR-016's structure/intent split).

**Driver.** The motivating use case is a **concept record (`CPT`, IMP-244 part 1 /
SL-197) saying annotated things about entities**:
```
link CPT-001 references SL-128 --role concerns --descriptor "frames estimation as attention burden"
```
`references` with `role = concerns` = "aboutness/relevance, source → any numbered
entity"; the `descriptor` is the specific statement. Descends from **IMP-244
part 2**.

The nuance is *free text with no graph meaning*: stored, round-tripped, rendered,
searchable, but inert to validation, inference, graph traversal/effects,
reciprocity, and edge identity.

## 2. Current State

- **`RelationLabel`** (`relation.rs:45`) is the closed outbound vocabulary; the
  **`RELATION_RULES`** table (`relation.rs:374`) keys `(source, label, role)` to
  five axes: `target`, `tier`, `link` policy, `inbound_name`, and — the load-bearing
  precedent — **`degree_bearing: bool`** (`relation.rs:363`), a per-row column
  gating an optional per-edge facet (`Degree`, SL-176) on exactly the `Fulfils` row.
- **`Degree`** is the working template for "optional, per-edge, identity-excluded,
  per-row-gated facet":
  - Rule column `degree_bearing`; gate in `validate_link` (`:1447`) — a degree on a
    non-bearing row bails (`DegreeNotApplicable`), symmetric to `RoleNotApplicable`.
  - Serialized on the `Tier::One` `[[relation]]` row only when present (`:1186`),
    load-bearing for diff stability.
  - **Excluded from edge identity** — identity is the `(label, role, target)` triple
    (`row_matches`, `:1265`); `append_relation_row` (`:1136`) matches the triple,
    then Noops on identical degree / hard-rejects a differing degree
    ("unlink to change").
  - Read struct `RelationEdge.degree` (`:746`); **not** carried on the hydrated
    `CatalogEdge` (degree's inbound render re-scans via `inbound_degree_index`,
    `relation_graph.rs:308`).
- **`references:concerns` rule** (`relation.rs:417`): `sources` **hand-enumerated**
  `[SL, RFC, ISS, IMP, CHR, RSK, IDE, ASM, DEC, QUE, CON, EVD, HYP]` (NOT `RECORD`
  splatted — the comment notes the record tail is *manually* kept in sync);
  `target: AnyNumbered`; `tier: One`; `link: Writable`. Authored via
  `link → append_edge`. This is the sole descriptor home (see §7 D2).
- **Search** (`search.rs:124`) indexes `title + body` (the `.md`) **only**; relation
  TOML is unindexed. `scan_catalog` (`search.rs:323`) hands the search path the full
  `Catalog` — `.entities` **and** `.edges` — so a source-join is feasible.
- **Hydrated catalog** (`catalog/hydrate.rs`): `CatalogEdge` (`:129`) carries
  `label/role/target/origin` — **no facet payload**.

## 3. Forces & Constraints

- **ADR-004** — relations stored outbound-only; reciprocity derived. Descriptor
  rides the outbound row; no authored inbound descriptor.
- **ADR-010** — contract/write-seam unified, storage bespoke. Descriptor rides the
  `Tier::One` `[[relation]]` seam.
- **ADR-016** — closed label + closed role. Descriptor adds **no** label or role;
  it is a third, non-semantic attribute (the ruling: a new *label* is for
  structure/graph-effect/inbound change, a new *role* for intent — descriptor is
  neither).
- **Behaviour-preservation gate** — shared entity-engine machinery; existing
  relation suites stay green unchanged (absent descriptor ⟹ prior behaviour).
- **House lint** — Strings via `Vec<String>+concat`, no `push_str(&format!(…))`.

## 4. Guiding Principles

- **Ride the `Degree` seam; do not invent a parallel facet mechanism.** Descriptor
  is `Degree` re-skinned as free text — same rule column, gate, serialization,
  identity-exclusion, conflict-reject shape.
- **Source-agnostic mechanism.** Descriptor is a property of the `references:concerns`
  *edge*, legal for every source the rule already admits (SL, backlog, records,
  and — once SL-197 wires it — `CPT`). This slice does not touch the source-set.
- **Additive, not migratory.** No rename, no data migration, no template change.

## 5. Proposed Design

### 5.1 System Model

Descriptor is an **optional free-text cell on a `Tier::One` `[[relation]]` row**,
admissible on **exactly one rule: `references` with `role = concerns`**. It is
inert to all graph semantics and excluded from edge identity. It surfaces on
**outbound** relation render and in the **search index**.

**Scope boundary (adversarial-pass F0).** IMP-244 also listed `contextualizes`,
`related`, `interactions`. All are excluded:
- `contextualizes` — `sources: &[CM]`, authored as **concept-map DSL** lines
  (`source > rel > target`, `concept_map.rs:1550`), a wholly separate write path
  the `link`/`append_edge` seam never touches. A descriptor there needs a
  concept-map DSL grammar change → **follow-up**, not this slice.
- `related`, `interactions` — **symmetric** edges (`inbound_name == label.name()`);
  a directed "descriptor" is a category error. `interactions` keeps its existing
  free-text `notes`; `related` gets no free text (YAGNI). See §7 D2.

### 5.2 Interfaces & Contracts

**Rule table** — one per-row column (mirrors `degree_bearing`):
```rust
// RelationRule (relation.rs:340)
pub(crate) descriptor_bearing: bool,   // true ONLY on the references:concerns row
```
Because `RELATION_RULES` keys per `(source, label, role)`, granularity is free:
the `references:concerns` row sets `true`; the sibling `references:implements` /
`references:originates_from` rows set `false`. So `link … references … --role
implements --descriptor …` resolves the implements row → `descriptor_bearing`
false → rejected. No special-casing.

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
    anyhow::bail!(
        "`{label_str}` does not take a descriptor; \
         descriptors are for `references --role concerns` edges");   // F5: hint the home
}
```

**CLI** — `--descriptor <TEXT>` on the `link` subcommand (`commands/cli.rs`). Raw
text (no parse-enum helper); validated **non-empty** (trim; empty/whitespace →
`"descriptor must be non-empty"`). The memory-source branch (`run_link`, `:89`)
**refuses** `--descriptor`, symmetric to its `--role` refusal (memory→entity
descriptors deferred).

**Read + hydrate**:
```rust
RelationEdge.descriptor: Option<String>   // relation.rs:735, beside .degree
CatalogEdge.descriptor: Option<String>    // catalog/hydrate.rs:129 — NEW facet on the hydrated edge
```

### 5.3 Data, State & Ownership

On-disk shape — new cell written **only when present** (load-bearing for diff
stability); facets cluster before `target`, `target` stays the trailing anchor:
```toml
# concept CPT-001 says an annotated thing about SL-128 (the driver)
[[relation]]
label = "references"
role = "concerns"
descriptor = "frames estimation as attention burden"
target = "SL-128"
```

Ownership: descriptor is authored on the **source** entity's outbound row (ADR-004),
mutated only through `link` / `unlink`. `descriptor_bearing` is owned by
`RELATION_RULES`.

### 5.4 Lifecycle, Operations & Dynamics

- **Add** — `link CPT-1 references SL-y --role concerns --descriptor "…"` → gate →
  append row with the cell.
- **Re-link, same triple, identical descriptor** → `Noop` (no rewrite/mtime churn).
- **Re-link, same triple, differing descriptor** → **hard reject**
  (`"already references SL-y with descriptor=…; unlink to change"`). `unlink` +
  relink is the explicit change path.
- **Unlink** — matches `(label, role, target)`; descriptor ignored (`:371`).
- **Facet-conflict is generalized** — `append_relation_row`'s same-triple branch
  compares *both* facets: `Noop` iff both equal, else bail naming the differing
  facet. Correct regardless of overlap; `descriptor_bearing ∩ degree_bearing = ∅`
  merely means at most one is ever `Some` (a simplification, not a dependency).
- **Render** — descriptor prints on the **outbound** edge in `inspect`/`show`
  (`references(concerns) SL-128 — "…"`). **No** inbound render, **no**
  `inbound_descriptor_index` (diverge from degree — descriptor is verbose
  source-authored prose; duplicating on every target's inbound list bloats).
- **Search** — `hydrate` populates `CatalogEdge.descriptor`; the search doc-build
  (`search.rs:323–336`) groups `Catalog.edges` by source and appends each entity's
  outbound descriptors to its lex text (`entity_lex_doc` gains the annotation
  string).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** — descriptor excluded from edge identity: `row_matches` /
  `relation_row_find` key on `(label, role, target)` only.
- **INV-2** — `descriptor_bearing` is `true` on exactly the `references:concerns`
  row, `false` elsewhere; disjoint from `degree_bearing`. (Disjointness simplifies
  the conflict check but is not required for its correctness.)
- **INV-3** — a `[[relation]]` row serializes `descriptor` iff non-empty-present.
- **INV-4** — existing relation behaviour unchanged when no descriptor is authored.
- Edge cases: empty/whitespace rejected; multi-line/special chars round-trip via
  `toml_edit::value`; descriptor on a non-bearing role/label or memory source
  rejected; descriptor never affects reciprocity, priority, traversal, dedup.

## 6. Open Questions & Unknowns

- **OQ-1 (resolved)** — reject vs ignore illegal placement → **reject** (house policy).
- **OQ-2 (resolved)** — append-conflict on differing descriptor → **reject** (degree precedent).
- **OQ-3 (resolved)** — scope → `references:concerns` only; contextualizes/related/interactions excluded (§5.1, F0).
- **OQ-4 (deferred)** — memory-source descriptors → refused this slice.
- **OQ-5 (cross-slice, SL-197)** — the driver needs `CPT` added to the
  `references:concerns` `sources` array. SL-197 assumes CPT auto-inherits via
  `RECORD`, but the source-set is hand-enumerated (§2) — **SL-197 must add CPT
  explicitly**. Flagged to SL-197; not this slice's touch-site.

## 7. Decisions, Rationale & Alternatives

- **D1 — descriptor rides the `Degree` seam.** Alt: bespoke free-text mechanism —
  rejected (parallel implementation; `Degree` proves the exact shape).
- **D2 — split annotation by directedness; scope to `references:concerns`.** The
  `inbound_name` axis distinguishes directed (`concerned by`) from symmetric
  (`related`, `interactions`). Descriptor = directed-edge free text. `contextualizes`
  (also directed) is excluded not by directedness but by write path (concept-map
  DSL, F0). Alt considered + rejected: one `descriptor` everywhere, migrating
  `interactions.notes` — flattens a real distinction, forces a breaking `--notes`
  rename + 8-row migration.
- **D3 — outbound-only render** (diverge from degree's inbound index) — verbose
  prose vs terse marker.
- **D4 — descriptor on the hydrated `CatalogEdge`** (not degree's re-scan side-path)
  — consumer-neutral home; search + any future consumer project from one structure.
  *Novelty risk:* no `degree` precedent on `CatalogEdge` (§8 R3).
- **D5 — `related`/`interactions`/`contextualizes` excluded** — symmetric (notes) or
  different write path (DSL); each a follow-up if demanded.

## 8. Risks & Mitigations

- **R1 — signature churn** across `validate_link`/`append_edge`/`run_link` +
  constructor sites. *Mitigation*: compile-driven; `descriptor: None` mirrors the
  `degree: None` defaults at `:768/780`.
- **R2 — search doc-build coupling** (group `Catalog.edges` by source).
  *Mitigation*: `CatalogEdge.descriptor` keeps it a pure projection; no re-parse.
- **R3 — `CatalogEdge.descriptor` is novel plumbing** — `degree` is not on
  `CatalogEdge`, so no precedent; `hydrate` must newly thread the raw cell.
  *Mitigation*: phase-plan confirms `hydrate` has the raw row in scope before
  committing to D4; fall back to a source-keyed re-scan (degree's pattern) if not.
- **R4 — behaviour-preservation regressions** in shared relation machinery.
  *Mitigation*: existing suites green unchanged; new VTs additive.
- **R5 — verbose/multiline descriptor render noise**. *Mitigation*: outbound-only;
  truncate/wrap is a phase-plan concern.

## 9. Quality Engineering & Validation

VT set (mirrors the `Degree` VTs at `relation.rs:3106+`):

| VT | Behaviour |
|---|---|
| VT-1 | Round-trip: `references:concerns` row with `descriptor="…"` writes and reads back |
| VT-2 | Identity-excluded: same triple + identical descriptor → Noop; differing → reject |
| VT-3 | Gate: `--descriptor` on a non-bearing row (`references:implements`, `supersedes`, `related`, `fulfils`) → reject with the home-hint |
| VT-4 | Accept: `references --role concerns --descriptor …` |
| VT-5 | Empty/whitespace descriptor rejected |
| VT-6 | Unlink triple-match: descriptor ignored |
| VT-7 | On-disk order `label / role / descriptor? / target` |
| VT-8 | Render: descriptor on outbound `inspect`; absent on inbound |
| VT-9 | Search: entity with a descriptor edge found by descriptor text; `CatalogEdge.descriptor` populated |
| VT-10 | Bearing pin: `descriptor_bearing` true exactly on `references:concerns`; disjoint from `degree_bearing` |

Behaviour-preservation (VA/VH): existing relation + catalog suites green unchanged.

## 10. Review Notes

**Internal adversarial pass (integrated):**
- **F0 (CRITICAL, integrated)** — `contextualizes` is `CM`-source and authored via
  concept-map DSL, not `link`/`append_edge`. Descriptor via the Degree seam cannot
  reach it. **Resolution:** narrowed `descriptor_bearing` to `{references:concerns}`;
  contextualizes descriptor → follow-up (concept-map DSL grammar).
- **F0b (cross-slice)** — SL-197's "CPT auto-inherits References(concerns) via
  RECORD" is false (hand-enumerated sources). Flagged to SL-197 (OQ-5).
- **F2 (resolved)** — search path holds the full `Catalog` (entities+edges); join
  feasible.
- **F3 (risk raised)** — `CatalogEdge.descriptor` novelty → R3.
- **F4 (doc, fixed)** — disjointness reworded as simplification (INV-2, §5.4).
- **F5 (fixed)** — reject message hints the descriptor home.
- **F6 (phase-plan)** — single-source the `"descriptor"` cell-name string if the
  codebase does so for `"degree"`/`"role"`/`"label"` (currently literals — follow
  suit; low priority, STD-001).

**External:** GPT inquisition pending (per handoff).
