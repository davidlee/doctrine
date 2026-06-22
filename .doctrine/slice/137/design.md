# Design SL-137: Corpus-level relation query verb — list edges by label, target, source-kind

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

The only relation read surface is `inspect <ID>` — per-entity, one hop. There is
no corpus-wide query: "show all `governed_by` edges", "which entities reference
ADR-001?", "how is the relation vocabulary used across the corpus?". The data
already exists, fully typed, in the hydrated `Catalog` (`catalog::hydrate`) — but
the only corpus-level reader is `catalog graph` (raw developer JSON). RFC-001's
thesis names this exact gap: the graph is rich on the inside, thin on the outside.

This slice adds two read verbs over that existing data — `relation list`
(filterable edge enumeration) and `relation census` (per-label distribution with
a target-resolution health breakdown). Pure consumption: zero new modelling, no
write path, no new disk I/O.

## 2. Current State

- `Catalog.edges: Vec<CatalogEdge>` is the corpus edge set, built once by
  `scan_catalog` (`catalog::hydrate`). Each `CatalogEdge` carries:
  - `source: CatalogKey` — `Numbered(EntityKey)` (has a `prefix`) or `Memory(String)`.
  - `label: CatalogEdgeLabel` — `Validated(RelationLabel)` (closed vocabulary,
    numbered entities) or `Raw(String)` (memory free-text labels).
  - `target: EdgeTarget` — `Resolved(CatalogKey)` | `UnresolvedRef{raw}` |
    `UnvalidatedText{raw}`.
  - `origin: EdgeOrigin`.
- The edge target is classified once, at hydration, against the scanned key set
  (`classify_target`) — this design re-reads that classification, never recomputes it.
- `inspect` is the only relation reader for entities; it prints scan-degradation
  diagnostics to stderr and renders one entity's 1-hop neighbourhood.
- `link`/`unlink` are top-level write verbs in `commands/relation.rs`
  (`run_link`/`run_unlink`).
- The read spine (`listing`): `Column<R>` + `render_columns` + `json_envelope` +
  `Format`. The corpus-projection precedent is `coverage_view` (defines
  `Column<CoverageRow>`, renders via the spine; bespoke projection, shared render).

## 3. Forces & Constraints

- **ADR-001 (layering):** leaf ← engine ← command, no cycles. The command reads an
  engine projection; the projection reads `catalog` (engine) + `listing`/`relation`
  (leaf).
- **ADR-004 (outbound-only, reciprocity derived):** edges are stored outbound only.
  `--target ADR-001` ("who references ADR-001") reads the outbound store and filters
  by target — no reciprocity computation, no inbound index needed.
- **ADR-010 (relation vocabulary):** the closed label set is `RELATION_RULES` /
  `RelationLabel`; memory labels are free-text `Raw` (mem.pattern.link.memory-label-fork).
- **Pure/imperative split:** projection/filter is pure over `&Catalog`; the only
  impurity (root discovery, scan, stdout/stderr) lives in the command shell.
- **Behaviour preservation:** read-only consumer — `catalog`/`relation`/`listing`
  internals are untouched, so their suites stay green unchanged.
- **Scope discipline (slice non-goals):** no transitive walk (SL-138), no graph
  export, no write path.

## 4. Guiding Principles

- Re-project existing data; do not re-model or re-classify it.
- A query returns "empty" rather than erroring on an unmatched filter — an empty
  result is a valid answer (no hard validation of filter values).
- Two orthogonal axes get two orthogonal controls; neither is hard-wired.
- Machine-readable JSON carries the full resolution axis even when the CLI default
  hides it behind a flag.
- Ride the shared render spine; do not fork table/JSON formatting.

## 5. Proposed Design

### 5.1 System Model

```
commands/relation.rs   (command)  RelationCommand{List,Census}; run_relation_list/_census
        │  root::find → scan_catalog → (print Error diags to stderr) → project → render → stdout
        ▼
relation_query.rs      (engine, NEW)  pure projection+filter+render over &Catalog
        ▼
catalog::hydrate (engine)  ·  listing (leaf)  ·  relation (leaf: RelationLabel)
```

`relation_query.rs` is a new engine-tier module (sibling to `relation_graph.rs`,
which owns the per-entity inspect projection; this one owns the corpus-flat query).
It is pure — `&Catalog` in, rows/strings out; no clock, rng, git, or disk.

The clap `RelationCommand { List, Census }` enum is added to `commands/relation.rs`
beside the existing link/unlink verbs; `cli.rs` registers a top-level `Relation`
subcommand (decision D1). `link`/`unlink` remain top-level (asymmetry accepted, D1).

### 5.2 Interfaces & Contracts

CLI:

```bash
doctrine relation list [--include-memory] [--label NAME]
                       [--target REF] [--source-kind PREFIX] [--unresolved]
                       [--format table|json] [--json]
doctrine relation census [--include-memory] [--format table|json] [--json]
```

`--include-memory` admits memory edges (the `Raw`-label / `MEM`-source population);
default is numbered entities only (the closed `RelationLabel` vocabulary). Because
hydration makes `Validated ⟺ numbered source` and `Raw ⟺ memory source` invariant,
"memory edges only" is `--include-memory --source-kind MEM` (D2).

Engine (`relation_query.rs`):

```rust
/// Target-resolution axis — shared by the list `state` field and census columns.
#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum TargetState { Resolved, Unresolved, FreeText }

fn target_state(t: &EdgeTarget) -> TargetState; // Resolved→R, UnresolvedRef→U, UnvalidatedText→F
fn target_display(t: &EdgeTarget) -> String;    // Resolved→canonical; else raw verbatim
fn source_kind(k: &CatalogKey) -> &str;         // Numbered→prefix; Memory→"MEM"

struct ListFilter {
  include_memory: bool,         // false (default) → numbered/Validated edges only
  label: Option<String>,        // exact match on label name()
  target: Option<String>,       // canonical-normalised match on target_display() (D6)
  source_kind: Option<String>,  // uppercased; exact match on source_kind()
  unresolved: bool,             // keep only state != Resolved
}

#[derive(Serialize)] struct RelationRow { source: String, label: String, target: String, state: TargetState }
#[derive(Serialize)] struct CensusRow  { label: String, count: usize, resolved: usize, unresolved: usize, free_text: usize }

fn project_list(cat: &Catalog, f: &ListFilter) -> Vec<RelationRow>;
fn project_census(cat: &Catalog, include_memory: bool) -> Vec<CensusRow>;
fn render_list(rows: &[RelationRow], fmt: Format, opts: RenderOpts) -> anyhow::Result<String>;
fn render_census(rows: &[CensusRow], fmt: Format, opts: RenderOpts) -> anyhow::Result<String>;
```

### 5.3 Data, State & Ownership

No new persistent state. The engine borrows `&Catalog` and produces owned row
vectors. The command owns the impure resources (root, scan result, stdout/stderr).
Label admission reads `CatalogEdgeLabel`; the target axis reads `EdgeTarget` —
both already populated by hydration. `source_kind` reads `EntityKey.prefix` for
numbered keys and emits the literal `"MEM"` for memory keys (the `CatalogKey`
identity boundary — mem.pattern.catalog.catalogkey-identity-boundary).

### 5.4 Lifecycle, Operations & Dynamics

**`project_list`** — iterate `cat.edges`; keep an edge iff ALL active axes admit it
(AND across axes): `include_memory` gate (false ⇒ drop `CatalogEdgeLabel::Raw`
edges) → `--label` matches `name()` → `--source-kind` matches `source_kind()` →
`--target` matches `target_display()` (canonical-normalised, D6) → `--unresolved`
⇒ `target_state != Resolved`. Map survivors to `RelationRow`. Sort `(label, source
canonical, target)` — the `target` key is the tie-breaker so duplicate
`(label, source)` rows (one source may hold several edges of one label to distinct
targets) order stably (F4c). Deterministic — no clock/rng.

**`project_census`** — filter `cat.edges` by the `include_memory` gate only; group
by label `name()`; per group accumulate `count` and per-`TargetState` tallies. Emit
one `CensusRow` per label. Sort `(count desc, label asc)`. Invariant:
`count == resolved + unresolved + free_text`.

**Render** — `Column<RelationRow>` (`source │ label │ target │ state`) and
`Column<CensusRow>` (`label │ count │ resolved │ unresolved │ free_text`) arrays;
`listing::render_columns` for table, `listing::json_envelope("relation" /
"relation-census", rows)` for JSON. Empty rows → empty string (header suppressed).

**Command shell** — `scan_catalog` → print only `Severity::Error` diagnostics to
stderr → resolve `RenderOpts` (`color` via `tty::stdout_color_enabled`, term width)
→ project → render → stdout.

Diagnostics policy (F5, corrected). Two distinct diagnostic classes ride
`scan_catalog`, and they are NOT symmetric:
- **classification diagnostics** — `UnresolvedRef`→Warning, `UnvalidatedText`→Info
  (`catalog/scan.rs`). The edge IS still emitted, so it is recoverable per-row via
  `--unresolved` (the dangling case) and counted in the census `unresolved`/
  `free_text` columns. Suppressing these from stderr loses no completeness.
- **edge-dropping diagnostics** — empty memory relation label/target emit Warning
  then `continue` (`catalog/hydrate.rs`), so the edge never enters `Catalog.edges`.
  These are NOT recoverable via `--unresolved`/census. To avoid a silently truncated
  view, the shell counts edge-dropping Warnings during scan and, when any fired,
  prints a single summary line to stderr (`N edge(s) dropped — run \`doctrine
  validate\` for detail`) — a bounded signal, not the ~1000-edge per-row flood
  (VT-11).

This is parity with `inspect`, not a divergence: `inspect` consumes
`relation_graph::scan_entities`, whose scan path emits `Severity::Error` only
(`commands/inspect.rs`), so `inspect` is already effectively Error-only on its own
path. The earlier framing ("diverges from `inspect`, which prints every diagnostic")
was factually wrong — `inspect` has no Warning/Info stream to print (F1).

`--target` is normalised before matching: when the input parses as a canonical
ref (`integrity::parse_canonical_ref`) it is compared in canonical form (so
`ADR-1` matches the stored `ADR-001`); otherwise the raw string is matched
verbatim (D6). **Memory targets match by UID only.** A Resolved memory target
displays as its memory UID (`CatalogKey::canonical()`), because hydration resolves
authored memory-key *aliases* through `mem_key_map` into `CatalogKey::Memory(uid)`
(`catalog/hydrate.rs`). `--target` does NOT re-expand an alias, so an alias input
falls to the verbatim branch and will not match a UID-displayed edge — pass the UID
(F3, VT-12).

### 5.5 Invariants, Assumptions & Edge Cases

- This surface does NOT ride `listing::build`/`retain`/`ListArgs` — its filters are
  not the substr/regex/status/tag set, so the `validate_statuses` opt-in trap
  (mem.pattern.listing.validate-statuses-is-opt-in) does not apply.
- Default (no `--include-memory`) excludes every memory `Raw`-label edge.
  `--include-memory` admits them; "memory only" = `--include-memory --source-kind
  MEM`. `--source-kind MEM` alone yields nothing without `--include-memory`.
- `drift` is a `Validated` label with an always-`UnvalidatedText` target — it counts
  under census `free_text` by design, not as a fault.
- No filter-value validation: an unknown `--source-kind`/`--label`/`--target`
  yields zero rows, never an error.
- `--source-kind` input is upper-cased before comparison (prefixes are upper-case).
- JSON always carries `state` (list) / the full breakdown (census), independent of
  any CLI default.
- Catalog symlink double-count is already handled at the scan walk
  (mem.pattern.entity.corpus-walk-skip-slug-symlink) — this consumer inherits the
  deduped edge set.
- **Scope: validated LIVE edges only (F2).** `scan_catalog` builds edges from the
  tier-1 reader, which drops off-table *illegal* `[[relation]]` rows before any edge
  exists (`relation::tier1_edges` discards the `_illegal` partition;
  `relation_graph` must re-read raw TOML for `validate` precisely because
  "scan_entities drops the illegal rows"). So a hand-edited illegal relation row is
  invisible to `relation list`/`census` and emits no diagnostic. This is by design:
  illegal rows are the sole province of `doctrine validate` (the `IllegalRow`
  consumer). This verb queries the legal, live edge set — it is not a structural
  linter, and the F1 dropped-edge summary covers only the empty-field Warning class,
  not illegal rows.

## 6. Open Questions & Unknowns

None blocking. Resolved during design:
- OQ-1 (namespace) → D1.
- OQ-2 (census "active labels" semantics) → two orthogonal axes, D2.
- OQ-3 (target-state granularity) → 2-way `--unresolved` bool, 3-way deferred (D3).

## 7. Decisions, Rationale & Alternatives

- **D1 — `relation { list, census }` namespace; link/unlink stay top-level.**
  Matches the slice's stated shape, minimal scope. Alternative B (group link/unlink
  under `relation`, deprecate top-level) churns existing verbs/tests/ADR-cited
  command shapes — its own cohesion slice if ever wanted. Alternative C (flat
  top-level `relations`) reads worse. Asymmetry accepted; reversible.
- **D2 — two orthogonal axes; provenance is a single bool.** Axis A
  label-provenance = `--include-memory` (default off → numbered/`Validated` only);
  Axis B target-resolution = `--unresolved` (list) / always-on breakdown (census).
  Rejected: hard-excluding raw/unvalidated (the slice's first phrasing) — a default
  that can't be opened is wrong. Rejected tri-state `--labels validated|raw|all`
  (F1): the hydration invariant `Validated ⟺ numbered`, `Raw ⟺ memory` makes the
  `raw` value redundant with `--source-kind MEM`, and `--labels` collided visually
  with the `--label` filter. A bool carries the only non-redundant choice ("include
  the memory population or not").
- **D3 — `--unresolved` is a 2-way bool, list-only.** The census breakdown
  (`resolved/unresolved/free_text` columns) subsumes it for census, so census takes
  no `--unresolved`. A 3-way `--target-state` (split dangling vs free-text) is
  deferred — the split is already in catalog diagnostics + `doctrine validate`, and
  the raw target string is visible per row.
- **D4 — census always shows the resolution breakdown.** It is free (every edge is
  already classified) and turns census into a per-label health profile. `Error`
  severity is entity-level (no edge/label to attribute) ⇒ stays on stderr, never a
  column.
- **D5 — new engine module `relation_query.rs`.** Cohesion: corpus-flat query is a
  distinct responsibility from `relation_graph.rs` (per-entity inspect) and from
  `relation.rs` (the leaf vocabulary). Pure, mirroring `coverage_view`.
- **D6 — `--target` is canonical-normalised.** When the input parses as a canonical
  ref it matches in canonical form (`ADR-1` ≡ `ADR-001`); otherwise raw verbatim.
  Removes the exact-match brittleness (was R2).

## 8. Risks & Mitigations

- **R1 — name confusion across `relation.rs` (leaf), `relation_graph.rs`
  (per-entity engine), `relation_query.rs` (corpus engine).** Mitigation: module
  doc-comments state each one's responsibility; `_query` vs `_graph` distinguishes
  flat-corpus vs per-entity.
- **R2 — RESOLVED → D6.** `--target` exact-match brittleness removed by
  canonical normalisation.
- **R3 — scope creep toward transitive / inbound indexing.** Mitigation: explicit
  non-goal; `--target` is a flat filter over the outbound store, not a reciprocity
  index.
- **R4 — RESOLVED → D2/(b).** Provenance/source-kind redundancy (the hydration
  invariant `Validated ⟺ numbered`, `Raw ⟺ memory`) collapsed the tri-state to the
  `--include-memory` bool.

## 9. Quality Engineering & Validation

Pure projection ⇒ tested over a seeded `Catalog` (`catalog::test_helpers` /
`from_scanned`), no disk. VT:

- **VT-1** `target_state`/`target_display` over all three `EdgeTarget` variants.
- **VT-2** `source_kind`: Numbered→prefix, Memory→`"MEM"`.
- **VT-3** `project_list` filters: `--label`, `--target`, `--source-kind` (incl.
  `MEM`), AND-composition — plus a case exercising all FOUR axes simultaneously
  (`--label` + `--target` + `--source-kind` + `--unresolved`, `include_memory` on),
  proving the AND narrows to the single intended row (F4b).
- **VT-4** `include_memory` gate: default excludes `Raw` (memory) edges; with the
  flag they appear. Corner: `--source-kind MEM` WITHOUT `--include-memory` → empty,
  since the gate drops the memory population first (F4a).
- **VT-5** `--unresolved` keeps only state ≠ Resolved.
- **VT-6** list sort `(label, source, target)`; the `target` tie-breaker orders two
  edges sharing `(label, source)` to distinct targets deterministically (F4c); empty
  result → empty string.
- **VT-7** `project_census` breakdown: `count == resolved+unresolved+free_text`;
  `drift` all-free_text; sort `(count desc, label asc)`.
- **VT-8** census `include_memory` honored (raw labels only with the flag).
- **VT-9** JSON shapes: list `{source,label,target,state}`; census
  `{label,count,resolved,unresolved,free_text}` under `json_envelope`.
- **VT-10** e2e black-box CLI golden: `relation list`/`census` wire end-to-end on a
  fixture corpus (clap registration + command shell smoke).
- **VT-11** diagnostics: a malformed-entity fixture emits an `Error` line on
  stderr; a dangling-ref edge (classification Warning) adds NO per-row stderr line;
  an empty-field memory relation row (edge-dropping Warning) DOES emit the single
  `N edge(s) dropped` summary line (F1).
- **VT-12** `--target` normalisation: `--target ADR-1` matches a stored `ADR-001`
  resolved edge (D6); and a resolved memory target matches by its UID, not by an
  authored key alias (F3).

Behaviour preservation: no edits to `catalog`/`relation`/`listing` internals ⇒
existing suites green unchanged.

## 10. Review Notes

### Internal adversarial pass (2026-06-22)

- **F1 (OPEN — UX call):** `--labels` (provenance) sits one letter from `--label`
  (filter) — a human footgun. Deeper: hydration makes `Validated ⟺ numbered source`
  and `Raw ⟺ memory source` invariant, so the provenance axis overlaps
  `--source-kind` (the `raw` value ≡ `--source-kind MEM`). Options:
  (a) keep tri-state, rename to `--label-kind validated|raw|all`;
  (b) collapse to a bool `--include-memory` (default = validated/numbered only),
      "raw only" expressed as `--include-memory --source-kind MEM`;
  (c) keep as authored (`--labels`), accept the collision.
  **RESOLVED → (b)** (user, 2026-06-22): `--include-memory` bool, default off.
  Drops the collision and the redundancy; "memory only" = `--include-memory
  --source-kind MEM`. D2 updated; `LabelScope` enum removed from §5.2.
- **F2 (FIXED):** `RenderOpts` (color + term width) must be resolved in the command
  shell — now explicit in §5.4.
- **F3 (FIXED → D6):** `--target` canonical normalisation kills exact-match
  brittleness.
- **F4 (FIXED → VT-11):** diagnostics behaviour now has a verification case.
- **F5 (SUPERSEDED → see external pass X1):** the "Error-only is a divergence from
  `inspect`" framing was wrong; corrected to parity in §5.4.
- Layering re-checked: command → `relation_query` (engine) → `catalog`/`listing`/
  `relation`. No up-calls, no cycle (ADR-001 clean). ADR-004 honoured (`--target`
  reads the outbound store; no reciprocity compute). ADR-010 vocabulary read-only.

### External adversarial pass — codex/GPT-5.5 via RV-139 (2026-06-22)

Inquisition (RV-139, `--raiser inquisitor`). Codex cleared three lines under
cross-examination, with code evidence: the load-bearing D2 invariant holds
(`Validated` labels are constructed only in the numbered-source loop,
`hydrate.rs:224`; `Raw` only in the memory loop, `hydrate.rs:285` — no mixed
constructor path); the "free-text that parses as a canonical ref" collision cannot
arise (`classify_target` routes any parsable ref to Resolved/UnresolvedRef, never
`UnvalidatedText`); ADR-001 layering + ADR-004 outbound-only stand. Four charges
sustained and reconciled into the design:

- **X1 (F-1, major → fixed §5.4):** Error-only rationale rested on a false premise.
  `inspect` consumes `relation_graph::scan_entities` (Error-only scan path), so it is
  already effectively Error-only — not a noisy "prints every diagnostic" foil. And
  "Warning/Info are surfaced by `--unresolved`/census" is false for *edge-dropping*
  Warnings (empty memory rows `continue` before the edge exists). Fixed: rationale
  rewritten to parity; shell now prints a bounded dropped-edge summary line.
- **X2 (F-2, major → fixed §5.5):** illegal hand-edited `[[relation]]` rows are
  dropped by the tier-1 reader with no diagnostic — invisible to the query. Scoped
  explicitly as a validated-live-edge verb; illegal rows remain `doctrine validate`'s
  province.
- **X3 (F-3, minor → fixed §5.4/D6):** `--target` was underspecified for resolved
  memory targets. Pinned: memory targets match by UID, not authored alias.
- **X4 (F-4, minor → fixed §9):** §9 overclaimed coverage. Added VTs for the
  `--source-kind MEM` empty case, the full four-axis AND, and a `(label, source,
  target)` tie-breaker for stable sort.
