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
     heavyweight, dispositioned review is the first-class **RV `reviews` label**.
     **On F-5 (F6, honest framing):** the review-in-prose cases SL-145 deferred here are
     *aboutness* — "this work concerns that artefact" — and are captured by
     `references(concerns)`; B does **not** launder a distinct evaluative edge into
     `concerns`. A dedicated non-RV evaluative role is **YAGNI** (zero live instances,
     and minting it would be the speculative vocabulary this slice rejects, cf.
     `exclusive_with`); if a genuine evaluative-vs-relevance distinction ever earns its
     keep, it is added then. The information "loss" is therefore a distinction that was
     never instantiated, not a downgrade of existing data.

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
- **Role must ride as edge PAYLOAD through the graph, distinct from overlay keying
  (F1 — resolves the D3/D5 tension).** Cordage **overlay allocation stays label-keyed**
  (one `references` overlay; graph-effect/priority is consumer policy, D3/R5 hold). But
  role-derived inbound rendering needs the role at the projection layer, where today
  `InspectView.inbound` is keyed by `RelationLabel` only (`relation_graph.rs:566-586`)
  and `render_inbound` calls `inbound_name(*label)` with no role
  (`relation_graph.rs:703-716`); JSON inbound is likewise label-only
  (`relation_graph.rs:752-767`). Without change, inbound `implements` and `concerns`
  edges collapse into one bucket and the verb is unrecoverable. **Fix:** carry
  `role: Option<Role>` on the graph/projection edge data (`RelationEdge` →
  `CatalogEdge` → `InspectView`), re-key `InspectView.inbound` by `(label, role)`, and
  pass role to `render_inbound`/JSON. Overlay count is unchanged; only the rendered
  grouping gains the role dimension.

### 2.7 Surfaces — complete seam inventory (F5/F8)

Every seam below is **label-only today** and must thread role. (Codex F8: `search.rs:29-41`
is `GROUP_ALIASES`, a kind-selector alias table — the `"specs"` entry is a *kind group
name*, unrelated to the relation label; **removed from blast radius**.)

| seam | file:line | change |
|---|---|---|
| `CatalogEdge` / `CatalogEdgeLabel` | `catalog/hydrate.rs:42-55, 127-135, 263-268` | carry `role`; label-only enum + struct extended |
| inspect grouping/render | `relation_graph.rs:555-586` (`inspect_from`), `703-716` (`render_inbound`), `752-767` (JSON) | re-key `InspectView.inbound` by `(label,role)`; pass role to render + JSON (F1) |
| `relation list` / `census` rows | `relation_query.rs:91-109, 121-185, 188-226` | row label + grouping carry `(label,role)` |
| `validate_relations` lookup | `relation_graph.rs:361-368` | role-aware `lookup`; new role-class `IllegalRow` findings |
| `link` / `unlink` verbs | `commands/relation.rs:17-31, 34-45, 282-321` | add `--role`; thread to `validate_link`/`append_edge`/`remove_edge` |
| per-kind `show` / `show --json` | `slice.rs:1530-1612`, `backlog.rs:1367-1494`, `lazyspec.rs:218-240` | named `specs`/`requirements` JSON fields disappear (see below) |
| web graph edge label | `catalog/graph.rs` | edge label shows the role verb |

**`show --json` schema change (the consumer-facing one).** `slice.rs`/`backlog.rs`/
`lazyspec.rs` build **named** `specs` / `requirements` fields via `targets_for(tier1,
RelationLabel::Specs|Requirements)`. Removing the variants deletes those keys. P4 decides
the replacement: proposal — a `references` object keyed by role
(`{ implements: […], scoped_from: […], concerns: […] }`). This is load-bearing for the
slice/backlog `show --json` goldens; enumerate the affected goldens in P4.

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

No `unspecified` ever persists; every landed row carries a real role. This slice rewrites
its own `slice-149.toml` `specs SPEC-018` row to `references(implements) SPEC-018` (and
`related RFC-003` → `references(concerns) RFC-003`, since RFC-003 is the deliberation this
slice is *about*, not a symmetric peer) as part of the pass.

**Execution mechanism — "atomic" is not a mechanism (F3).** SPEC-018 forbids dual-read,
and current readers (`read_block` `relation.rs:562-616`; `validate_relations`
`relation_graph.rs:374-398`) reject any unknown label / off-table row. So a
**partially**-rewritten corpus is invalid under both old and new code — there is no valid
intermediate state. The migration therefore MUST:
1. build the **full** rewrite in memory — every `[[relation]]` row across the whole
   corpus transformed in one pass (deterministic rows + the hand-dispositioned ambiguous
   rows resolved up front);
2. apply it as a **single atomic swap** (write all files, then commit) — never
   row-by-row through the live `link` verb;
3. run `validate` **only after** the complete rewrite is applied.

The new code (role-aware parser) and the rewritten corpus land in the **same commit**;
there is no commit in which code and corpus disagree. Plan's vehicle choice (gated test
vs throwaway bin) must honour this all-or-nothing shape.

---

## Section 3 — Verification alignment

**Behaviour-preservation gate — the machinery/content split.** The entity-engine
*machinery* (generic seam, read/write dispatch, `outbound_for`, `validate_relations`)
stays behaviour-preserving — those suites green unchanged. The *vocabulary content*
changes deliberately — goldens for the collapsed `specs`/`requirements` edges update with
the migration. The design's proof obligation: an explicit list of which goldens
legitimately change vs which must stay byte-identical.

Tests to change / add:

- **Lockstep (VT family) — correct attribution (F4):** VT-1 (enum incl `References`,
  minus `Specs`/`Requirements`); **VT-2 `sources_match_shipped_accessors`
  (`relation.rs:1052`)** is the *source-set* audit — update to the role-bearing rows;
  **VT-4 `reader_emitted_labels_equal_table_labels_per_source` (`relation_graph.rs:1635`)**
  is the *exact-coverage* invariant — extend from per-`label` to per-`(label, role)` (the
  fully-populated fixture authors one edge of every legal `(label, role)`); VT-3
  `inbound_name` identical per `(label, role)`. New: `legal_roles` reachability;
  `(source, label, role) → TargetSpec` gate
  goldens.
- **Validation:** `MissingRole`, `IllegalRole`, `RoleNotApplicable`, role-target mismatch
  refused; corpus `validate` flags a hand-edited bad/missing-role `references` row as
  `IllegalRow`.
- **Storage round-trip:** author `references(implements)` → `[[relation]] label role
  target` → read back → `inspect` outbound `references(implements)` + target inbound
  "implemented by"; `unlink` matches the `(label, role, target)` triple; a label-only
  edge serializes with no `role` key.
- **Migration — the oracle must verify role ASSIGNMENT, not just edge survival
  (F2 supersedes the earlier AR-2 framing).** Edge-set preservation alone is *insufficient*:
  a bug mapping every `specs` → `references(concerns)` would still preserve the
  `(source,target)` multiset while silently corrupting exactly the classification this
  slice exists to set. The oracle is therefore:
  1. **Exact expected `(source, target) → (label, role)` per deterministic row** — the
     migration emits its planned mapping; the test asserts each deterministic row landed
     at its mapped `(label, role)` (not merely that the edge survived).
  2. **A reviewed disposition artifact for every ambiguous row** (SL→SPEC
     implements-vs-concerns; non-peer `related`) — each with the chosen role + one-line
     rationale; the test asserts each ambiguous edge matches its recorded disposition.
  3. **Edge-count + `(source,target)` multiset preservation** — *secondary* sanity check
     (zero added/dropped), not the primary proof.
  *Plus:* render is **not** byte-identical (it changes by design — `specs` →
  `references(implements)`; inbound `"specs"` → `"implemented by"`); after-migration render
  goldens assert the **new** expected vocabulary; a storage-level post-check guards on-disk
  row order (render launders it — SPEC-018 concern); `validate` is clean (no `IllegalRow`,
  no dangler regression); the disposition artifact (SL→SPEC + ambiguous `related`) is
  committed as evidence with
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
5. **P5** — out-of-band migration: full in-memory corpus transform (F3) + ambiguous-row
   disposition artifact + single-shot apply + role-assignment oracle (F2) + round-trip
   verification (hard cut, same commit as the parser).
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
- **`scoped_from`-vs-`part_of` boundary (F7 — accepted residual, no extra tightening).**
  `scoped_from` ships in B; `part_of` arrives in Axis D. The boundary is held by
  definition + the target-kind gate (backlog-only targets), **not** a structural
  invariant. Decision (user): keep `scoped_from`; do **not** add special tightening now;
  clean up any origin-vs-containment violations when Axis D builds `part_of`. The
  contamination surface is narrow (backlog-targeted decomposition only) and the role earns
  its place immediately (the ~13 SL→backlog origin edges).

---

## Adversarial review (external pass — codex/GPT, integrated)

Hostile external review of the committed draft. Findings F1–F8; all code claims verified
against the source before integration.

- **F1 [BLOCKER] role-derived inbound vs label-keyed graph** — `InspectView.inbound`,
  `render_inbound`, JSON inbound are label-only (`relation_graph.rs:566-586, 703-716,
  752-767`); one `references` overlay would collapse `implements`/`concerns` inbound.
  *Fixed:* §2.6 — role rides as edge payload through the graph/projection; overlay stays
  label-keyed; inbound re-keyed `(label,role)`.
- **F2 [BLOCKER] migration oracle too weak** — edge-set preservation can't see role, the
  thing being changed. *Fixed:* §3 — exact `(source,target)→(label,role)` per
  deterministic row + reviewed disposition artifact per ambiguous row; preservation
  demoted to secondary.
- **F3 [BLOCKER] "atomic" is not a mechanism** — no valid partially-rewritten state under
  no-dual-read. *Fixed:* §2.9 — full in-memory transform → single-shot apply → validate
  after; parser + corpus in one commit.
- **F4 [MAJOR] missing table invariants + mis-attributed tests** — *Fixed:* §3 — VT-2
  (`relation.rs:1052`, source-audit) and VT-4 (`relation_graph.rs:1635`, exact-coverage)
  both updated; add "one rule per `(source,label,role)`" + "`(source,label)` wholly
  roleful or roleless" invariants.
- **F5/F8 [MAJOR/MINOR] §2.7 incomplete + bogus `search.rs`** — *Fixed:* §2.7 rewritten as
  a line-level seam inventory; `search.rs` (a `GROUP_ALIASES` kind-selector,
  `search.rs:29-41`) removed from blast radius.
- **F6 [MAJOR] `concerns` swallows `reviews`** — *Fixed (framing):* §1 corollary — B does
  not launder an evaluative edge into `concerns`; F-5's cases are aboutness; a distinct
  evaluative role is YAGNI (zero instances), not a data downgrade.
- **F7 [MAJOR] `scoped_from`/`part_of` boundary unenforced** — *Accepted residual* (user):
  keep `scoped_from`, defer cleanup to Axis D (Carried opens, above).

---

## Adversarial review (internal pass — integrated)

Hostile self-review of the draft. Findings AR-1..AR-6 integrated above; recorded here for
the audit trail.

- **AR-1 — migration is per-edge, not kind-deterministic.** Grounded empirically: live
  `related`=74 (vs RFC snapshot 48→60), dominated by RFC→* `concerns`; RFC→RFC / ADR→ADR /
  SL→SL need per-edge judgment. *Fixed:* §2.9 now re-censuses live, treats the P1 artifact
  as stale reference, and widens hand-triage beyond SL→SPEC.
- **AR-2 — verification oracle was wrong** (render-byte-identity). *Superseded by F2:* the
  edge-set-preservation fix was itself too weak; §3 now asserts role assignment directly.
- **AR-3 — `show`/`show --json` projection ripple (missed code impact).** *Refined by F5/F8:*
  `slice.rs`/`backlog.rs`/`lazyspec.rs` are real; `search.rs` was a false positive
  (kind-alias). §2.7 carries the corrected inventory.
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
