# CHR-024 Prep 3: Research synthesis

> Five concise syntheses covering the relation contract, entity landscape, known
> friction points, preliminary findings (F-1 through F-7), and web graph builders.
> Dense enough that an architect can load all five in one context alongside the
> audit checklist. Produced from Prep 1 (artifact index) and Prep 2 (code map).
>
> RFC authoring phase should read this first, then the audit checklist (§Scope in
> the backlog chore body), then the artifact index / code map as needed.

---

## 1. The relation contract

### What RELATION_RULES says

The entire relation model is driven by one const table in `src/relation.rs`:
`RELATION_RULES: &[RelationRule]`. Each row declares a `(sources, label)` pair
and the five axes that govern it — `target` (forward validation), `tier` (storage
shape), `link` (verb admission), and `inbound_name` (derived-reciprocal render text).

The vocabulary has **20 outbound labels** that span 7 semantic classes:

| Class | Labels | Examples |
|---|---|---|
| Composition / lineage | `descends_from`, `parent`, `members`, `interactions` | SPEC→PRD, SPEC→SPEC, PRD/SPEC→REQ |
| Authorisation / governance | `governed_by`, `owning_slice`, `consumes` | SL→ADR, REC→SL, PRD→PRD |
| Work → artefact | `specs`, `slices`, `requirements`, `drift`, `reviews`, `revises` | SL→PRD, BACKLOG→SL, REV→SPEC |
| Peer association | `related`, `contextualizes`, `shapes`, `spawns` | ADR↔ADR, CM→any, REC→any |
| Replacement | `supersedes` (+ reciprocal `superseded_by` carve-out) | SL→SL, ADR→ADR |
| Free-text / external | `decision_ref` | REC→free-text DEC cite |
| Epistemic (knowledge record) | `shapes`, `spawns` | ASM→SL, QUE→IMP |

The table is declared in `RelationLabel` enum declaration order; a lockstep test
(`enum_ord_matches_relation_rules_label_order`) ensures the two never diverge. Every
variants distinct label appears exactly once in the `RELATION_RULES` declaration
sequence, and within a label, multiple source-rows sit adjacently at that label's slot.

### How it drives the five consumers

1. **`relation::tier1_edges`** — the generic `[[relation]]` block parser. Every
   per-kind `relation_edges` accessor calls this for its tier-1 edges, then
   concatenates its own tier-2 typed edges. The parser splits legal edges from
   `IllegalRow` findings; illegal rows are dropped from reads but surfaced by
   `validate`.

2. **`catalog::scan::outbound_for`** — the KINDS-driven dispatch over 19 arms (one
   per authoring kind). Each arm delegates to its kind's `relation_edges` accessor.
   This is THE single read seam — every consumer (inspect, priority, catalog graph,
   validate) rides this.

3. **`relation_graph::build_relation_graph_from`** — the cordage graph builder.
   Allocates one overlay per resolvable label (every `TargetSpec != Unvalidated`).
   Table-derived (R2-M4): iterates `RELATION_RULES`, no hardcoded label const.
   A new resolvable label gets an overlay automatically.

4. **`relation::validate_link` / `check_target_kind`** — the `link` verb's forward
   legality gates. Three refusal layers: unknown label → list legal labels for source;
   illegal for source → list legal labels; link-policy not Writable → name the owning verb.

5. **`relation_graph::validate_relations`** — the `validate` relation walk. Reports
   danglers (validated label targets that don't resolve), illegal rows (hand-edited
   off-table pairs), and supersession drift (stored vs derived `superseded_by`).

### The tier model

- **Tier 1** (`Tier::One`): uniform `[[relation]]` rows in the entity's TOML. Read
  generically via `read_block`, written via `link`/`unlink`. ~14 labels live here.
- **Tier 2** (`Tier::Typed`): bespoke per-kind structures — `members.toml`,
  `interactions.toml`, `descends_from` scalar, `revises` from `[[change]]` rows.
  Authored only through their own typed verbs.
- **Tier 3**: free-text (classification facets like tags, contact). Not relations —
  no entity references.

### The write seam

`link`/`unlink` are the generic tier-1 write verbs. The flow:
1. Parse source ref via `integrity::parse_canonical_ref` → `(KindRef, id)`
2. Validate `(source_kind, label)` via `relation::validate_link` → refusal or `&RelationRule`
3. Validate forward target:
   - `Unvalidated` labels (drift, decision_ref): target is free text, no resolution
   - Resolvable labels (`Kinds`/`SameKind`/`AnyNumbered`): target must resolve
     (`ensure_ref_resolves`) AND pass kind check (`check_target_kind`)
4. Append/remove via `relation::append_edge`/`remove_edge` — edit-preserving
   `toml_edit` with F1 trailing-typed-table defence, idempotent (no-op on
   duplicate/absent).

Separate typed verbs handle tier-2 edges: `supersede` for `supersedes`/`superseded_by`,
`spec req add` for `members`, `review <ID>` for `reviews`, `revision change add` for `revises`.

---

## 2. The entity landscape

### What the KINDS table says

`src/integrity.rs` defines `KINDS: &[KindRef]` — 21 numbered entity kinds in canonical
order. Each `KindRef` carries a `kind` descriptor (with `prefix`, `dir`, `stem`) and
optionally a `state_dir` (only for slice today).

### Kind-by-kind: who authors what, who reads what

| Prefix | Kind | Authors (outbound labels) | Reads (inbound from) | Notes |
|---|---|---|---|---|
| **SL** | Slice | `specs`, `requirements`, `supersedes`, `governed_by`, `related` | `superseded_by`, `governs`, `related`, `slices` (from backlog), `owning_slice` (from rec) | Only work kind with `requirements`; also hosts `needs`/`after` dep/seq |
| **ADR** | ADR | `supersedes`, `related` | `superseded_by`, `related`, `governs`, `revises` | Governance kind; `supersedes` is lifecycle-only (written by `supersede` verb) |
| **POL** | Policy | `supersedes`, `related` | `superseded_by`, `related`, `governs` | Same lifecycle pattern as ADR |
| **STD** | Standard | `supersedes`, `related` | `superseded_by`, `related`, `governs` | Same lifecycle pattern as ADR |
| **PRD** | Product spec | `descends_from`, `parent`, `members`, `governed_by`, `consumes` | `specs`, `descends_from`, `consumed_by`, `governs`, `revises` | `members` links PRD→REQ; `consumes` PRD→PRD |
| **SPEC** | Tech spec | `descends_from`, `parent`, `members`, `interactions`, `governed_by`, `consumes` | `specs` (from SL/backlog), `parent`, `descends_from`, `interactions`, `governs`, `revises`, `shaped_by` (from records) | Richer edge set; `interactions` carries free-text `type` annotation |
| **REQ** | Requirement | *(none — edge target only)* | `requirements` (from SL), `members` (from PRD/SPEC) | Purely a target; no outbound relations |
| **ISS** | Issue | `specs`, `slices`, `drift` | `governed_by` (ABSENT — F-1), `related` (ABSENT — F-2), `slices`, `spawned_by` (from records) | Backlog kind; `drift` is target-unvalidated |
| **IMP** | Improvement | `specs`, `slices`, `drift` | Same as ISS | Backlog kind |
| **CHR** | Chore | `specs`, `slices`, `drift` | Same as ISS | Backlog kind |
| **RSK** | Risk | `specs`, `slices`, `drift` | Same as ISS | Backlog kind |
| **IDE** | Idea | `specs`, `slices`, `drift` | Same as ISS | Backlog kind |
| **RV** | Review | `reviews` (TypedVerbOnly — single `[target].ref`) | *(no standard inbound — targets don't list reviewers)* | Derived `active`/`done` status |
| **REC** | Reconciliation record | `owning_slice`, `decision_ref` | *(limited inbound)* | `decision_ref` is target-unvalidated |
| **REV** | Revision | `revises`, `originates_from` (both TypedVerbOnly) | `revises` (inbound on targets) | Edges from `[[change]]` rows |
| **RFC** | RFC | *(none — currently prose-only, edges via `related` on SL side)* | `originates_from` (from REV), `related` (from SL) | Passive participant |
| **CM** | Concept map | `contextualizes` (via `link`, but ISS-041: invisible to `outbound_for`) | `contextualizes` (inbound) | Currently returns `Ok(vec![])` from `outbound_for` |
| **ASM** | Assumption (knowledge record) | `shapes`, `spawns`, `supersedes` (lifecycle-only) | `shaped_by`, `spawned_by`, `governed_by` (as source of governance) | Newest seam (SPEC-019) |
| **DEC** | Decision (knowledge record) | `shapes`, `spawns`, `supersedes` (lifecycle-only) | Same as ASM | Note: `DecisionRef` label is NOT the same as DEC kind |
| **QUE** | Question (knowledge record) | `shapes`, `spawns`, `supersedes` (lifecycle-only) | Same as ASM | |
| **CON** | Constraint (knowledge record) | `shapes`, `spawns`, `supersedes` (lifecycle-only) | Same as ASM | |

### What each kind can be the target of

Some kinds serve predominantly as targets (REQ, RFC), while others (SL, ADR, SPEC) are
both active authors and frequent targets. The full target set per label is defined by
`TargetSpec` in `RELATION_RULES` — a kind that's absent from all `TargetSpec::Kinds`
lists can never be a structured relation target (e.g. backlog kinds cannot be targeted
by `governed_by` — F-1).

### Memory entities

Memory is NOT in `KINDS` — it's a named-kind corpus (`mem_<uid>` dirs). Memory entities
use `CatalogEdgeLabel::Raw` (free-form label strings), not the closed `RelationLabel`
vocabulary. Memory edges are surfaced by `--include-memory` in `relation list`/`census`
and by a separate `memory_inspect_view`. They never ride the `relation_graph::inspect`
path or the priority engine overlays.

---

## 3. Known friction points

Each item below is an open backlog item that signals modelling strain.

### ISS-041: Concept-map `contextualizes` edges writable via `link` but invisible to read paths

The `contextualizes` label is in `RELATION_RULES` as `LinkPolicy::Writable` for CM
source-kind, and `doctrine link CM-001 contextualizes ADR-005` succeeds. But CM kind's
`outbound_for` arm returns `Ok(Vec::new())` — the concept map builder reads from its
own DSL, not from the relation graph. The edge is *written* into a `[[relation]]` row,
but *never read* by any read path. This is a confirmed code-level disconnect at the
`catalog::scan::outbound_for` dispatch: the CM arm is explicitly stubbed.

**Impact:** The semantic graph and `inspect` never show `contextualizes` edges from
concept maps. The only render path is the concept map's own markdown/Mermaid output.
Any analysis that assumes `link`-writable = `inspect`-readable breaks for CM.

### IMP-149: Ambiguous `slices` relation kind on backlog items

The `slices` label on backlog items (BACKLOG→SL) conflates two readings:
- "this backlog item was implemented by this slice" (post-hoc)
- "this backlog item should be implemented by this slice" (planning)

The label carries no axis for intent vs completion. Combined with F-6 (no
completeness semantics), an agent or validator cannot distinguish "this backlog
item is fully addressed by this closed slice" from "this backlog item needs
further slices."

### IMP-138: Relation-transitive walk for `inspect`

`inspect` shows one-hop direct relations only (I2 in the design). There is no way
to ask "what entities are transitively governed by POL-001 through chain of
ADR→POL governed_by edges?" or "what specs does this ADR govern through the full
closure?" This limits the relation model's expressiveness for impact analysis.

### IMP-141: `doctrine validate` relation visibility

The current validator checks danglers, illegal rows, and supersession drift. It does
NOT check: whether every entity has appropriate edges (e.g. "a done slice should have
at least one `specs` edge"), whether coverage is consistent ("this backlog item is
still open but both its `slices` are closed"), or whether legal but suspicious
patterns exist (e.g. circular `governed_by`).

### ISS-046: `backlog needs` rejects SL-prefixed slice targets

The `needs` verb validates targets against work-kind types, but SL-prefixed targets
may be rejected because `backlog::kind_from_prefix` doesn't recognise "SL". This
signals a gap between the `needs`/`after` validation (which lives in `dep_seq.rs`)
and the cross-kind `outbound_for` dispatch.

### IMP-053: Record↔record associative relation class for SPEC-019

The knowledge-record spec (SPEC-019) identified a need for record-to-record
associative relations (`informs`/`bears-on`) that don't fit the existing
`supersedes` lifecycle or the `shapes`/`spawns` work→artefact pattern. This
unresolved requirement is why `relation-vocabulary.md` self-identifies a
"class 6" epistemic gap.

### IDE-015: Bridge concept map to relation graph

The inverse of ISS-041: rather than making CM edges visible to the relation graph,
this proposes making relation edges visible IN concept maps — so a concept map could
auto-render all `contextualizes` edges from its DSL as graph edges. Either way, the
CM↔relation graph seam needs reconciliation.

### IMP-134: Extend tagging to all appropriate entity types

Another axis of the "coverage completeness" theme: not all kinds that should carry
tags do, and the tagging verb (`doctrine tag`) may not cover all kinds uniformly.
Requires checking which kinds have `tags` in their scaffold template and render path.

---

## 4. Preliminary findings (F-1 through F-7)

These seven findings were identified during chore creation when the chore's own
relationships were interrogated. They are provisional — the RFC may confirm,
refine, or reject them.

### F-1: `governed_by` excludes backlog kinds

The `governed_by` label's source-set is `{SL, PRD, SPEC, CM, ASM, DEC, QUE, CON}` —
backlog kinds (ISS/IMP/CHR/RSK/IDE) are absent. A backlog item governed by an ADR
has no structured way to express it. Direct evidence: CHR-024 (this review) cannot
say `governed_by ADR-004` or `governed_by ADR-010` except as prose. Extending the
source-set to include BACKLOG kinds would fix it.

### F-2: `related` excludes backlog kinds (except through SL/RFC)

`related` has two rules: GOV→SameKind and {SL, RFC}→AnyNumbered. No BACKLOG→AnyNumbered
or BACKLOG→BACKLOG rule exists. A chore that needs to express `related ISS-041` or
`related IMP-149` has no structured peer-association outlet — only `drift` (free-text,
unvalidated). This directly impedes cross-referencing between related backlog items.

### F-3: `specs` on backlog items conflates three distinct semantics

The single `specs` label collapses "emitted from" (this work was generated from this
spec), "implements" (this work delivers this spec), and "peer/bears-on" (this work is
associated with this spec without delivery relationship). An agent reading `specs
SPEC-018` cannot distinguish whether the item is implementing the spec, reviewing it,
or was scoped from it. Options: mint distinct labels, extend `related` to BACKLOG, or
add a payload facet.

### F-4: CHR-024 cannot link to SPEC-018 in a semantically meaningful way

Compound of F-1/F-2/F-3: the only label CHR-024 could use (`specs`) forces
"implements" semantics, which is inaccurate — this chore reviews SPEC-018, it
doesn't implement it. A parallel ambiguity exists on the `slices` axis (IMP-149).

### F-5: No `reviews` label for backlog kinds

`reviews` exists only for RV kind (TypedVerbOnly, review→target). A backlog item
that reviews or audits a spec (like CHR-024) cannot say so as a structured edge.
If `reviews` were open to SL/BACKLOG sources (with appropriate semantics), this
chore would use it.

### F-6: No completeness/completion semantics on spec→slice coverage

The model has no way to express partial satisfaction. A backlog item links to spec S
via `specs`. Slice SL-1 implements part of S and closes. If the backlog item spawned
SL-1, `/close` step 5 transitions the backlog item to done — but spec S needs both
SL-1 and SL-2. The model cannot distinguish "this slice fully satisfies this spec"
from "this slice addresses this spec but more work remains." There is no coverage
predicate, partial-satisfaction indicator, or cross-slice dependency on the `specs`
edge. Currently relies on LLM discernment — a brittle check.

### F-7: No work-entity decomposition model (hierarchy / epic pattern)

Neither slices nor backlog items can express decomposition. A broad slice cannot
declare `decomposes_to SL-011, SL-012`; an epic backlog item cannot declare
"contains IMP-051, IMP-052." The `supersedes` label replaces wholesale rather than
decomposing; `related` is peer association; `parent` is SPEC→SPEC only. No structured
way exists to track splits, validate that children are closed before the parent closes,
or display work-item hierarchy on any read surface.

---

## 5. Web graph builders

### Semantic graph (`src/catalog/graph.rs` → `src/map_server/routes.rs`)

**How it consumes relations:** Projects `CatalogGraph` from the hydrated `Catalog`.
Every `CatalogEdge` becomes a graph edge in the JSON response served at `GET /api/graph`.
Edges carry `CatalogEdgeLabel` which is the merged type: `Validated(RelationLabel)` for
numbered-entity edges and `Raw(String)` for memory-entity edges. The web map server
renders this as an interactive d3-force graph in the browser.

**What would break if the model changed:**
- Adding/removing a label: the `CatalogEdgeLabel` enum must gain/lose a variant.
  Serialized label strings change (the `.name()` of the new label).
- Changing tier-1 → tier-2: edges disappear from `[[relation]]` blocks, stop appearing
  in the scan, and vanish from the graph. Existing graph consumers see partial edges.
- Changing the memory label fork: if memory moves from `Raw` to `Validated`, the
  `CatalogEdgeLabel::Raw` variant may become unused or the merge point changes shape.
- Model changes that affect entity scan order (new/removed KINDS) change node index
  order in the graph response.

### Actionability graph / priority engine (`src/priority/graph.rs`)

**How it consumes relations:** Builds a separate cordage `Graph` from the same
`relation_graph::scan_entities` + `dep_seq_for`. One overlay per resolvable
`RelationLabel`, plus `needs`/`after` dep/seq overlays. The `needs` overlay is
`Reject` (blocking), `after` is `Evict` (soft sequence), reference overlays feed
consequence scoring. The `OrderSpec` is `[dep Along, seq Along]` — backlog-order
style with hard deps first, then soft.

**What would break if the model changed:**
- **New resolvable label**: automatically gets an overlay via `OverlayMap::build`
  (R2-M4), but the overlay has no policy — it's `Reject, Unbounded`, which means
  edges appear in the graph but have no ordering effect until the priority engineer
  assigns a `CyclePolicy`/`EdgeAttrs` per new label's semantics. A label meant to be
  blocking (like `needs`) would silently be non-blocking.
- **Changed source-set**: entities that newly author a label contribute edges to an
  existing overlay. If the overlay has blocking/unbounded semantics, the new edges
  affect priority rankings — potentially incorrectly.
- **Removed label**: its overlay persists in allocated state but receives no new
  edges (dormant). The `survey`/`next`/`blockers` read surfaces exclude empty overlays.
- **Label promoted from target-unvalidated to resolvable**: gets an overlay where
  previously it had none. `Drift` edges (currently dangling) suddenly resolve and
  affect priority — a breaking semantic change.

### Concept map (`src/concept_map.rs` → `src/map_server/markdown.rs`)

**How it consumes relations:** The concept-map builder reads from its own DSL
parsed from the concept map's markdown body — NOT from the relation graph. The DSL
defines nodes and edges with string labels (including `contextualizes` for relation
edges). `concept-map edge add` wraps `doctrine link` to write the `[[relation]]`
row, but the builder never reads it back.

**What would break if the model changed:**
- **The ISS-041 disconnect means the concept map builder is already independent of
  the relation model's read paths.** A label rename in `RELATION_RULES` wouldn't
  break the concept map builder — but it would break the `link` verb, which the
  `concept-map edge add` command wraps. So the concept map's WRITE path is coupled
  to the model (labels must match `RELATION_RULES`), but its READ path is not.
- **Fixing ISS-041** (making CM edges visible to `outbound_for`) WOULD couple the
  builder to the model's read paths. The concept map's own DSL would then be a
  secondary render concern; the primary edge source would be the relation graph.
- **Changing `contextualizes` label semantics** (e.g. adding target validation where
  currently it's `Unvalidated`) would affect which targets `concept-map edge add`
  accepts.

### Cross-cutting: what breaks everywhere

- **Changing `RELATION_RULES` source-sets** affects every kind's `relation_edges`
  output (via `tier1_edges`), every `link` validation, every overlay allocation, and
  the `validate` walk — coordinated update across 5 layers (vocabulary, read, write,
  priority, validate).
- **Adding a new label to the enum** requires: `name()` + `from_name()` arms, one or
  more `RELATION_RULES` rows, source-set + target + tier + link-policy, inbound_name,
  and one overlay allocation per resolvable label. If the label should affect priority,
  a cordage overlay policy decision.
- **Removing a label** requires the inverse: removing enum variant, removing
  `RELATION_RULES` rows, removing overlay (or leaving it dormant — `OverlayMap::build`
  won't allocate it). Existing authored edges with that label become `UnknownLabel`
  illegal rows. The `migrate` verb must rewrite or delete them first.

---

*Produced 2026-06-23 as CHR-024 Preparatory subtask 3. Next: RFC authoring phase.*
