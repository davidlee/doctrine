# SL-149 Design — References role grammar (RFC-003 Axis B)

<!-- Reference forms: entity ids padded (SPEC-018, ADR-010); doc-local refs bare
     (D1 decision, OQ-1 open question, R1 risk). -->

## Status

Design draft for the RFC-003 Axis B core: collapse the work→canon relation family into
one `references` label refined by a closed **role** dimension, re-keying target
validation from `(source, label)` to `(source, label, role)`. The ratifying ADR
(Section 1) is authored as part of this slice (routing decision: fold the ADR into
`/design`); no code precedes its acceptance.

Governing context already in canon: **ADR-004** (outbound-only, reciprocity derived),
**ADR-010** (unify contract + write seam, keep storage bespoke), **SPEC-018** (the
cross-corpus relation contract), **RFC-003** (the deliberation this slice ratifies).

## Decision ledger (locked)

| # | Decision |
|---|---|
| **D1 ADR** | New ADR (next id), **composes with** ADR-010 (no supersession); ratifies the three principles + boundary rulings (Section 1). |
| **D2 enum** | Role enum `{implements, scoped_from, concerns}` — closed, all directional. `reviews` **dropped** (folds into `concerns`; the first-class RV `reviews` *label* is untouched). |
| **D3 related** | `related` **stays its own symmetric-neutral label**; does not collapse. Symmetry changes inbound semantics → structural → label, not role (RFC's own deciding principle). |
| **D4 table** | Single `RELATION_RULES`; add a `role: Option<Role>` column; `references` row-explodes (one row per `(source-set, role)`). `lookup(source, label, role)`; one lockstep table. |
| **D5 inbound** | Role-derived inbound; `inbound_name` keyed `(label, role)`; VT-3 re-keyed. `supersedes`/`governed_by` keep label-keyed inbound + the ADR-004 `superseded_by` carve-out. |
| **D6 migrate** | Out-of-band deterministic one-time rewrite (**no shipped verb** — SPEC-018 precedent); deterministic map by `(source-kind, label, target-kind)`; hand-triage the ambiguous; hard-cut atomic with the code; **no persisted `unspecified`**. |

---

## Section 1 — The ratifying ADR

**ADR (next id): Relation intent as a closed role dimension — durable structure vs
contextual intent.** Composes with ADR-010 (contract + seam) and ADR-004
(outbound-only). Does **not** supersede them — ADR-010's storage/seam decisions remain
true; this ADR adds the intent dimension on top.

### Decision

1. **Structure/intent split.** A relation's durable *structural shape* (the **label**:
   sources, target-class, tier, graph-effect, inbound semantics) is separated from its
   *contextual intent* (a closed **role**). The work→canon family collapses into one
   `references` label refined by `{implements, scoped_from, concerns}`. Target validation
   re-keys from `(source, label)` to `(source, label, role)`. `governed_by`, `related`,
   `part_of`, `supersedes`, `exclusive_with` stay distinct labels.

   **Deciding principle (the ruling):** *separate LABEL* when the distinction changes
   **structure, graph-effect, or inbound semantics**; *ROLE facet* when it only refines
   the **intent** of an otherwise structurally-identical edge.
   - Corollary — `related` stays a label: it is **symmetric + neutral**; its reciprocal
     is itself, where every `references` role has a distinct directional inbound verb.
     Symmetry is an inbound-semantic difference → structural → label.
   - Corollary — `reviews` is **not** a role: it has no structural distinction from
     `concerns` (work → any target, evaluative/relevance intent, no gate); the
     heavyweight, dispositioned review is the first-class **RV `reviews` label**. The
     non-RV "review" outlet (F-5, deferred here by SL-145) is served by
     `references(concerns)`.

2. **Derivable-not-relational law.** Do not encode in the relation what is derivable
   from entity state or facets. **Coverage** (→ `validate` / `/close`), **temporal
   planned-vs-done** (→ target lifecycle status), and **altitude** (→ target facet) are
   *projections*, not labels. `slices` stays one edge; planned/done reads off status.

3. **Graph-effect in the consumer.** Whether an edge gates, evicts, or scores is
   consumer policy (priority overlay / `/close` / `validate`), not a property of the
   relation (consistent with IMP-047). Corollary (R5): cordage overlay allocation stays
   **label-keyed**, not `(label, role)` — `references` is one overlay-backed label
   regardless of role; cordage stays vocabulary-unaware.

### Scope boundaries (named, deferred — not decided here)

- Whether work can `implements` an **ADR** — `governed_by` stays the ADR relation for
  now; the likelier resolution is ADRs impose REQs and the REQ is the implemented thing.
- **Non-entity-target edges** (memory / file / glob / vec) — IMP-012, IDE-015.
- **Decomposition `part_of` + altitude facets** — Axis D sibling spec.
- **`exclusive_with`** and the **`influences` / relation-planes** question (directionality
  × valence) — modelled in RFC, demanded by no current edge.

---

## Section 2 — Model & code impact

All in `src/relation.rs` unless noted. Layering (ADR-001): `Role` + rules +
`lookup`/`validate` are leaf/engine; `link --role` is command; cordage stays unaware.

### 2.1 New `Role` enum (closed)

```rust
pub(crate) enum Role { Implements, ScopedFrom, Concerns }   // derive Ord for canonical order
// wire spelling: "implements" | "scoped_from" | "concerns"; name()/from_str round-trip
```

### 2.2 `RelationLabel`

Remove `Specs`, `Requirements`; add `References`. `Related`, `Slices`, `Drift`,
`GovernedBy`, `Supersedes`, … stay. VT-1 enum-declaration order shifts (code-level).

### 2.3 `RelationRule` — one new column

```rust
pub(crate) struct RelationRule {
    sources: &'static [&'static str],
    label: RelationLabel,
    role: Option<Role>,        // NEW — Some on references rows, None elsewhere
    inbound_name: &'static str,
    target: TargetSpec,
    tier: Tier,
    link: LinkPolicy,
}
```

### 2.4 `references` rows (replace every `specs`/`requirements` row)

```
references | [SL]                  | Implements  | "implemented by" | Kinds(SPEC,PRD,REQ) | One | Writable
references | [SL]                  | ScopedFrom  | "scoped into"    | Kinds(<backlog>)    | One | Writable
references | [SL,RFC,<backlog>]    | Concerns    | "concerned by"   | AnyNumbered         | One | Writable
```

- `implements` is **SL-only** — backlog items spawn slices that implement; they do not
  implement canon directly.
- `scoped_from` is **SL-only**, target the backlog kinds — "this slice was scoped from
  that idea/improvement." Kept strictly separate from `part_of` (Axis D containment).
- `concerns` rides **one wide source-set row**, target `AnyNumbered`. **Pin the exact
  source set in P2** (no `<backlog>` hand-wave): live census shows `{SL, RFC, ISS, IMP,
  CHR, RSK, IDE}` authoring concerns-shaped edges (RFC-002 alone authors related→IMP/SL/RFC);
  derive it from the migration population + SL-145's widened backlog set. `implements`
  and `scoped_from` are `{SL}` only.
- `inbound_name` per row (the role-derived inbound, D5). Wording settled in P2 (R4).

### 2.5 Edge / row shapes — thread role; identity = `(label, role, target)`

```rust
struct RelationEdge { label: RelationLabel, role: Option<Role>, target: String }
struct RelationRow  { label: String, role: Option<String>, target: String }  // serde skip_if None
// TOML:  [[relation]] label = "references"  role = "implements"  target = "SPEC-018"
//        [[relation]] label = "governed_by" target = "ADR-010"          # no role key
```

Idempotency (`append_edge`/`remove_edge`) matches the full triple. `read_block` parses
the optional `role`.

### 2.6 Functions re-keyed

- `lookup(source, label, role: Option<Role>) -> Option<&RelationRule>` — matches
  `(source ∈ sources, label, role)`. Label-only edges match the `role = None` row.
- New `legal_roles(source, label) -> impl Iterator<Item = Role>` — filter rows
  (drives the CLI error message + the `MissingRole`/`IllegalRole` gate).
- `validate_link(source, label, role)` — new error taxonomy:
  - `MissingRole` — a `references` link without `--role`.
  - `IllegalRole` — role not in `legal_roles(source, label)`.
  - `RoleNotApplicable` — `--role` given for a label-only edge (e.g. `governed_by`).
  - target-kind mismatch refused via `check_target_kind` reading the **role-keyed**
    `TargetSpec`.
- `inbound_name(label, role) -> &'static str`. VT-3 invariant → identical per
  `(label, role)`. `supersedes`/`governed_by` keep their existing label-keyed inbound
  (role `None`) and the ADR-004 carve-out is untouched.

### 2.7 Surfaces

- `CatalogEdge` / `CatalogEdgeLabel` (`catalog/hydrate.rs`) carry role.
- `inspect` (`commands/inspect.rs`, `relation_graph::render_*`): outbound renders
  `references(implements)`; inbound renders the role-derived name ("implemented by",
  "concerned by", "scoped into").
- `relation list` / `census` (`commands/relation.rs`): group by `(label, role)`;
  `relation_query.rs` row label/grouping handles the role.
- Web graph (`catalog/graph.rs`): edge label shows the role verb.
- **Per-kind `show` / `show --json` projections (AR-3 — not just rendering).**
  `slice.rs` (1533/1536/1599/1603), `backlog.rs` (1374/1489), and `lazyspec.rs` build
  **named** `specs` / `requirements` JSON fields via `targets_for(tier1,
  RelationLabel::Specs|Requirements)`; `search.rs:32` maps `"specs" → {PRD,SPEC}` in the
  search index. Removing the variants forces reworking these into a `references`-grouped-
  by-role projection — this **changes the slice/backlog `show --json` schema** (the
  `specs`/`requirements` keys disappear), which is load-bearing for goldens. Decide the
  replacement shape in P4 (proposal: a `references` object keyed by role, e.g.
  `{ implements: […], concerns: […] }`, plus a derived flat list for back-compat readers
  if any).

### 2.8 CLI

```
doctrine link <src> references --role implements <target>
doctrine link <src> governed_by <target>          # unchanged; --role here → RoleNotApplicable
doctrine unlink <src> references --role implements <target>   # matches the triple
```

### 2.9 Migration (out-of-band, not shipped — D6)

A one-shot deterministic pass (gated/ignored integration test or throwaway bin under the
slice — plan picks the vehicle; **not** a CLI verb), mapping per:

| current | shape | → |
|---|---|---|
| `specs` | SL→{SPEC,PRD} | `references(implements)` |
| `requirements` | SL→REQ | `references(implements)` |
| `specs` | IMP/RSK→canon | `references(concerns)` |
| `related` | RFC→* (bears_on) | `references(concerns)` |
| `related` | SL→backlog | `references(scoped_from)` |
| `related` | GOV↔GOV, SL↔SL (true peer) | **stays `related`** |
| `slices`, `drift` | — | **untouched** (temporal / out of B) |

**The map is per-edge, not purely kind-deterministic (AR-1).** Live census already
exceeds the RFC snapshot (`specs`=67, `requirements`=52, `related`=74; ~193 edges total,
vs the RFC's ~185), and `related` is dominated by RFC-001/RFC-002 → `concerns`. Kind
alone cannot separate `concerns` from a genuine symmetric peer on the `related`-source
rows. So the migration:
1. **re-censuses live** (`relation list`) at execution time — the P1 artifact
   (`.doctrine/state/chr-024/p1-classification.md`) is gitignored, snapshot-stale
   (counts grown 48→60→74), and is **reference only, not input**;
2. applies the kind-map where unambiguous;
3. **triages the residue by hand pre-commit** — which is wider than SL→SPEC: it includes
   (a) SL→SPEC `implements`-vs-`concerns`, and (b) every `related` row that is not
   clearly a GOV↔GOV / SL↔SL peer (RFC→* defaults to `concerns`, but RFC→RFC, ADR→ADR,
   and SL→SL get judged individually). Post-migration `related` ends up small (only the
   true peers).

No `unspecified` ever persists; every landed row carries a real role. Hard-cut, atomic
with the code change (SPEC-018 "no dual-read"). This slice rewrites its own
`slice-149.toml` `specs SPEC-018` row to `references(implements) SPEC-018` (and `related
RFC-003` → `references(concerns) RFC-003`, since RFC-003 is the deliberation this slice
is *about*, not a symmetric peer) as part of the pass.

---

## Section 3 — Verification alignment

**Behaviour-preservation gate — the machinery/content split.** The entity-engine
*machinery* (generic seam, read/write dispatch, `outbound_for`, `validate_relations`)
stays behaviour-preserving — those suites green unchanged. The *vocabulary content*
changes deliberately — goldens for the collapsed `specs`/`requirements` edges update with
the migration. The design's proof obligation: an explicit list of which goldens
legitimately change vs which must stay byte-identical.

Tests to change / add:

- **Lockstep (VT family):** VT-1 (enum incl `References`, minus `Specs`/`Requirements`);
  VT-2 exact-coverage now over `(label, role)` — per source kind the reader's emitted
  `(label, role)` set equals the table's; VT-3 `inbound_name` identical per `(label,
  role)`. New: `legal_roles` reachability; `(source, label, role) → TargetSpec` gate
  goldens.
- **Validation:** `MissingRole`, `IllegalRole`, `RoleNotApplicable`, role-target mismatch
  refused; corpus `validate` flags a hand-edited bad/missing-role `references` row as
  `IllegalRow`.
- **Storage round-trip:** author `references(implements)` → `[[relation]] label role
  target` → read back → `inspect` outbound `references(implements)` + target inbound
  "implemented by"; `unlink` matches the `(label, role, target)` triple; a label-only
  edge serializes with no `role` key.
- **Migration — the oracle is edge-set preservation, NOT render-byte-identity (AR-2).**
  Unlike SL-048's storage-only migration (render unchanged), here the *render changes by
  design* (`specs` → `references(implements)`; inbound `"specs"` → `"implemented by"`), so
  before/after render goldens cannot be the oracle. The preservation invariant is:
  **every pre-migration edge maps to exactly one post-migration edge with identical
  source + target; only `(label, role)` changes per the map; zero edges added or
  dropped.** Assert that as a structural diff over `relation list` (source,target multiset
  preserved). *Additionally:* after-migration render goldens assert the **new** expected
  vocabulary; a storage-level post-check guards on-disk row order (render launders it —
  SPEC-018 concern); `validate` is clean (no `IllegalRow`, no dangler regression); the
  triage dispositions (SL→SPEC + ambiguous `related`) are captured as evidence with
  per-row rationale.
- **Surfaces:** `inspect` mixed-roles + label-only golden; `relation list`/`census`
  grouped by `(label, role)`; web-graph edge label.
- **Determinism:** BTree ordering only; canonical `(label, role)` order = declaration
  order; no `HashMap` iteration on the relation path.

---

## Section 4 — Phasing, risks, carried opens

### Phasing sketch (plan refines; ADR is the gate)

1. **P1** — ADR authored + accepted (the ratification gate; no code before).
2. **P2** — `Role` enum + `RelationLabel` change + `references` rules + `lookup` /
   `legal_roles` + lockstep tests (red/green/refactor). Leaf/engine.
3. **P3** — storage (`RelationEdge`/`RelationRow`/`read_block`/`append`/`remove`) +
   `validate_link` taxonomy + `check_target_kind`.
4. **P4** — surfaces (`CatalogEdge`, `inspect`, `relation list`/`census`, web graph) +
   `link --role` CLI.
5. **P5** — out-of-band migration pass + triage + corpus rewrite + round-trip
   verification (hard cut).
6. **P6** — docs: rewrite SPEC-018 + `relation-vocabulary.md` to describe
   `references`/role; reconcile.

### Risks

- **R1** — stray string-matches on `"specs"`/`"requirements"` outside the rule table
  (grep before removal; do not confuse with the temporal `slices` label).
- **R2** — golden-churn volume; the machinery-vs-content split must be explicit so a
  reviewer can tell a deliberate vocabulary change from a regression.
- **R3** — SL→SPEC `implements`-vs-`concerns` triage is human judgment; capture rationale
  per row in the migration evidence.
- **R4** — inbound wording ("scoped into", "concerned by") is load-bearing for goldens;
  settle in P2 before surface goldens harden.
- **R5** — cordage overlay stays **label-keyed**, not `(label, role)` (graph-effect is
  consumer policy, ADR D3); confirms cordage stays vocabulary-unaware.

### Carried opens (deferred, named in the ADR)

- Work-`implements`-ADR (→ likely REQ-mediated).
- Non-entity-target edge (IMP-012, IDE-015).
- Axis D `part_of` + altitude facets (sibling spec).
- `related` symmetry / `influences` relation-planes (directionality × valence).
- **`scoped_from`-vs-`part_of` boundary** — B must not let `scoped_from` creep into
  structural containment (D's territory).

---

## Adversarial review (internal pass — integrated)

Hostile self-review of the draft. Findings AR-1..AR-6 integrated above; recorded here for
the audit trail.

- **AR-1 — migration is per-edge, not kind-deterministic.** Grounded empirically: live
  `related`=74 (vs RFC snapshot 48→60), dominated by RFC→* `concerns`; RFC→RFC / ADR→ADR /
  SL→SL need per-edge judgment. *Fixed:* §2.9 now re-censuses live, treats the P1 artifact
  as stale reference, and widens hand-triage beyond SL→SPEC.
- **AR-2 — verification oracle was wrong.** Draft leaned on before/after render byte-identity,
  but the render changes by design. *Fixed:* §3 oracle is now edge-set preservation
  (source+target multiset) + new-vocabulary render goldens + storage post-check.
- **AR-3 — `show`/`show --json` projection ripple (missed code impact).** `slice.rs`,
  `backlog.rs`, `lazyspec.rs`, `search.rs` hardcode named `specs`/`requirements` fields;
  removing the variants changes the `show --json` schema. *Fixed:* added to §2.7 + P4 with a
  replacement-shape proposal.
- **AR-4 — source sets were hand-waved.** *Fixed:* §2.4 pins them from live census, P2
  obligation.
- **AR-5 — `bears_on` → `concerns` renames the RFC's term.** Deliberate (the dialogue judged
  `bears_on` jargony/weak). *Action:* the ADR records the rename + rationale so the
  RFC↔ADR vocabulary divergence is explicit, not silent.
- **AR-6 — slice scope (`slice-149.md`) tells the older story** (lists `reviews`,
  `bears_on`, and `related`-collapses). *Action:* reconcile the scope doc to the locked
  ledger before planning (done in this pass).

Residual (accepted, not blocking): the `references` source set will need re-confirmation if
the corpus grows materially between design and P5 execution — the snapshot caveat applies;
P5 re-censuses live regardless.
