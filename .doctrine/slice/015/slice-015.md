# Spec entity v1: product + technical specs

## Context

Doctrine has product/technical specs as *intent only*: the `/spec-product` and
`/spec-tech` skills exist but both declare "Not yet structural — author prose
under `doc/*` by hand." The glossary already reserves the id schemes
(`PRD-001`, `SPEC-001`, `REV-001`) and two deferred design notes have already
worked a shape end-to-end:

- [`doc/spec-entity-spec.md`](../../../doc/spec-entity-spec.md) — the spec entity
  worked to serde structs: three subtypes, per-subtype facet sets,
  requirement-as-compound-key-row, FK validation = the registry's job.
- [`doc/entity-model.md`](../../../doc/entity-model.md) — the umbrella taxonomy
  (storage rule, entity/facet, generic-edge-table, three-layer Rust model,
  `status` not `state`).

This slice makes specs **first-class entities** — modelled on Spec-Driver's
product/technical specs but in the Doctrine way: simpler, cleaner, no less
powerful. It rides the shipped scaffold engine (`src/entity.rs`, SL-003) the way
ADR (SL-006) did — spec is the engine's next caller, not a reason to fork it.

**The seed note is partly superseded.** A pre-design reframe (recorded under
*Design Direction* below) overturns the note's two identity decisions —
requirement-owned-by-one-spec and requirements-never-move. The notes remain the
read-in for everything else; `/design` formalises the reframe and rewrites those
two sections of `spec-entity-spec.md`.

## Context Bundles & Sources

Where a `/design` agent should read in, grouped by authority:

**Governing design (committed, the seed):**
- [`doc/spec-entity-spec.md`](../../../doc/spec-entity-spec.md) — the worked spec
  entity: decomposition, requirement identity, serde structs, known risks, open
  questions. *Primary read — but its § Requirement identity and § Spec identity
  are superseded by the Design Direction here.*
- [`doc/entity-model.md`](../../../doc/entity-model.md) — umbrella taxonomy +
  adjudication (storage rule, entity/facet, edges, three-layer model).

**Supporting doc notes the spec note leans on (committed):**
- [`doc/relation-index.md`](../../../doc/relation-index.md) — the registry / FK
  validation; the *in-scope* `spec validate` rides its cache-independent pass,
  and its parsed edge-graph is where membership + the feature DAG live.
- [`doc/drift-spec.md`](../../../doc/drift-spec.md) — row↔prose orphan mitigation
  this slice inherits verbatim.
- [`doc/slices-spec.md`](../../../doc/slices-spec.md) — the directory-entity shape
  + reservation primitive reused, not reinvented.
- [`doc/reservation-spec.md`](../../../doc/reservation-spec.md) — the per-entity
  `mkdir` reservation namespace (now also a *requirement* caller — see below).
- [`doc/glossary.md`](../../../doc/glossary.md) — the reserved id schemes.

**Reference entities (code — the reuse seams, do not fork):**
- `src/entity.rs` — the kind-parameterised scaffold engine (SL-003, done).
- `src/adr.rs` + [`slice/006/design.md`](../006/design.md) — the worked
  "new entity rides the engine unchanged" precedent; `src/spec.rs` mirrors it.
- `src/slice.rs` — the other substrate caller.

**External source corpus (read-only, `/workspace/spec-driver/`):**
- `.spec-driver/tech/SPEC-110/` — the worked tech spec (the 7-block pathology).
- `.spec-driver/tech/SPEC-134/` — a `stub`-status tech spec.
- `.spec-driver/product/PROD-008/` + `requirements/FR-00{1,2,3}.md` — product
  spec with standalone-file requirements (the nascent freestanding-requirement
  idea this slice commits to).

**Local research (GITIGNORED, disposable — `scratch/`):**
- `scratch/spec-driver-schemas.local.md` — full Spec-Driver block schemas (275KB;
  the source the note's mappings derive from). Sample, don't dump.

## Design Direction (reframe — agreed pre-design)

The seed note models a spec as the aggregate root that *owns* requirement rows.
Discussion surfaced that this fights its own decomposition: the instant a feature
or another spec references `SPEC-A.FR-001`, that compound key is already a
cross-container address used as a stable handle. The model below makes that
honest. **These decisions are agreed but not yet locked** — `/design` formalises
them, `/inquisition` adversarially tests them, before any code.

The entity set shifts from `{ spec } + requirement-as-facet` to a small graph:

```
requirement   peer entity. reserved, durable immutable id; sticky local label;
              kind ∈ {functional, quality}. NOT a spec-owned facet row.
feature        grouping aggregate. own prose, priority, lifecycle. edges →
              requirements (membership), → slices (satisfaction), → features
              (dependency DAG). One kind + a `frame` seam.
spec           aggregate root (subtype product | tech). own identity + own prose
              + own facets, PLUS edges → requirements / features. A thin root
              with its own matter, not a pure view.
```

**D1 — requirement is a peer entity, not a spec facet.** Globally reserved
(`requirement/id/<n>`), its own directory/row; the `mkdir` reservation primitive's
next caller. `kind ∈ {functional, quality}` (functional ≈ feature-bearing;
quality ≈ non-functional).

**D2 — two-tier id, both immutable.**
- *Durable id* (global, reserved; candidate scheme `REQ-NNNN`, fixed in `/design`):
  the cross-ref handle. **Storage always uses it.**
- *Sticky local label* (`FR-001` / `NF-002`): human-facing, citable, **assigned
  once at add-time and never renumbered** (the `PHASE-NN` / `EN-` immutability
  rule), scoped to a spec membership, resolved through membership. CLI accepts it
  as input shorthand and resolves to the durable id.
- *Rejected:* pure-positional render (volatile — external citations rot on
  insert); nanoid-style opaque ids (hostile to humans).

**D3 — invariant: identity immutable, membership mobile.** Replaces the note's
"requirements never move / retire+reintroduce." Requirements relocate by
re-pointing membership edges; the durable id never changes. Global reservation
makes the note's "two branches add `FR-009` → silent duplicate" collision
*structurally impossible* (the claim arbitrates) — the risk is deleted, not linted.

**D4 — spec = thin aggregate root; feature = grouping aggregate.** Specs and
features both aggregate requirements via edges. Payload-bearing edges stay typed
facets (coverage: status+artefact; interactions: typed notes — a generic edge is
lossy); membership and dependency are generic edges.

**D5 — reading is a render concern; editing is component-level.** A readable flat
document is *emitted* (on demand, or optionally materialised to a committed
`*.rendered.md`): the spec's own block-elements + a rendered view of its
components. Editing stays at component granularity (atomic `spec req add` etc.).
**Render is a load-bearing v1 deliverable**, not a follow-up — it is the only
readable whole.

**D6 — grouping is one kind + a `frame` seam, not a family.** `feature` /
`capability` / `story` / `jtbd` are one structural kind differing only in prose
frame. Build the `frame` discriminator seam; implement a single default frame.
"Seam, not team." (This resolves the note's open "capability — entity or
grouping?" → entity, because it carries priority + lifecycle + DAG node.)

**D7 — design big, implement a coherent subset.** Design the full
`requirement + feature + spec` graph; implement the subset that ships specs
end-to-end. The subset is forward-compatible by construction (feature is all
edges over already-reserved requirements). The exact subset boundary is confirmed
in `/design`; see *Scope & Objectives*.

## Scope & Objectives

**Design (big):** the whole `requirement + feature + spec(product|tech)` graph
above, including the feature grouping layer, the dependency DAG, membership and
payload edges, the render contract, and the rewrite of the two overturned note
sections.

**Implement (coherent subset) — candidate v1, confirmed in `/design`:**

- **`requirement`** — peer entity: reservation (`requirement/id/<n>`), durable id
  + sticky local label, `kind ∈ {functional, quality}`, prose companion.
- **`spec new <product|tech>`** — scaffold the subtype fileset via the engine;
  per-subtype reservation (`spec/product/id/<n>`, `spec/tech/id/<n>`).
- **Membership edges** spec → requirement (and the typed payload facets a tech
  spec carries that survive the `/design` challenge).
- **`spec req add`** — atomic, **edit-preserving** (`toml_edit`) write of the
  requirement (durable id reserved, sticky label assigned) + its prose companion.
- **`spec show` / render** — reassemble identity + own facets + component
  requirements + inbound/outbound refs into one readable view (D5).
- **`spec validate`** — the FK-validation pass: every cross-entity ref resolves;
  dangling (`SPEC-TBD`-class) and duplicate refs flagged. Cycle detection arrives
  with the feature DAG (deferred), so v1 `validate` is FK-existence only.

**Deferred to a follow-up slice (designed here, built next):** the `feature`
entity, the dependency DAG + cycle validation, multi-frame grouping. Forward-
compatible — all edges over reserved requirements, no rework.

**Reuse, don't fork.** `src/spec.rs` and `src/requirement.rs` mirror
`src/adr.rs`/`src/slice.rs` over the shared `src/entity.rs` substrate; the
fileset-as-function descriptor supplies each subtype/kind's combination. Extract
only genuinely-shared substrate, as SL-006 did.

## Non-Goals

- **`feature` implementation + dependency DAG + cycle detection** — *designed*
  here, *built* in the follow-up slice (D7). v1 ships specs + requirements.
- **Multi-frame grouping** — the `frame` seam exists; only one default frame ships.
- **`revision` subtype** — home explicitly open (`entity-model.md` pushes back);
  resolved with the change/delta lifecycle.
- **The relation-index *cache*** — only the cache-independent FK-validation pass
  lands; the scale-gated index/cache half stays deferred.
- **Code↔spec sync adapters** (Spec-Driver's `sync`) — needs a code parser; later.
- **Coverage gap computation** — a registry query; v1 only *stores* coverage rows.
- **Spec-Driver corpus importer** — the migration note's job; not this slice.
- **Spec-Driver ceremony** — slot-system symlink trees, registry JSON, audit-gate
  automation, contract-variant dirs. The "simpler/cleaner" trim.
- **Spec lifecycle transitions / approval gating** — `status` hand-edited, ungated
  in v1, as slices/ADRs ship today.

## Risks, Assumptions & Open Questions

**Assumptions (carried):**
- `src/entity.rs` admits a new caller with a per-kind fileset descriptor with no
  engine change — supported by SL-003 (done) and SL-006's "rides the engine
  unchanged." Now exercised by *two* new callers (`spec`, `requirement`); exact
  API verified in `/design`.
- The `mkdir` reservation primitive scales to a *requirement* namespace (D1) — the
  same primitive slices/ADRs/specs use; requirement is one more caller.

**Risks:**
- **Entity-count at scale.** A 500k-LOC product has thousands of reserved
  requirement entities. `relation-index` budgets "thousands of edges, single-digit
  MB, fine"; requirement files are the same order, but now load-bearing — confirm
  the budget holds for *files*, not just edges.
- **Render is load-bearing (D5).** It is the only readable whole, so it is a
  correctness surface, not a nicety — must round-trip identity + facets + components
  faithfully.
- **Row↔prose orphans (self-drift).** Each component is a row + a `### id` prose
  heading; hand edits desync. Mitigation: atomic edit-preserving `add` +
  `list`-time orphan lint (inherited from `drift-spec`).
- **Behaviour-preservation gate.** Extending `src/entity.rs` touches shared
  machinery — existing slice/ADR/memory suites must stay green unchanged.

**Open questions (for `/design`):**
1. **Durable requirement id scheme** — `REQ-NNNN` global vs another shape; how the
   sticky local label is stored and resolved through membership (D2).
2. **Which tech facet tables survive** — the note's seven (requirements,
   capabilities, coverage, concerns, hypotheses, decisions, relationships): which
   are genuine payload facets vs collapsible into components + edges (the original
   "challenge the decomposition" mandate, now narrowed by the reframe).
3. **Exact v1 subset boundary** (D7) — whether any feature scaffolding is cheap
   enough to pull forward.
4. **Product facet set** — the light subtype's exact combination.

## Verification / Closure Intent

"Done" (v1 subset) is judged by:
- `spec new product` / `spec new tech` scaffold their filesets via the engine.
- `spec req add` reserves a durable requirement id, assigns a sticky local label,
  and writes row + prose companion atomically and edit-preservingly (round-trips
  without dropping comments / unknown keys).
- `spec show` / render reassembles identity + own facets + component requirements
  into one readable view.
- `spec validate` catches a deliberately-dangling cross-entity ref; passes a clean
  corpus.
- The full `requirement + feature + spec` model is designed and locked
  (`/inquisition`) even though feature is not built — the deferred layer is shown
  forward-compatible.
- Existing slice/ADR/memory suites green **unchanged** (behaviour-preservation).
- `cargo clippy` zero warnings (bins/lib); `just check` clean.
- TDD red/green/refactor throughout.

## Follow-Ups

- **`feature` entity + dependency DAG + cycle validation + slice-mapping** — the
  prime follow-up; the prioritisation/completion layer the reframe is built for.
- Multi-frame grouping (the rest of the `frame` family).
- `revision` subtype, once the change/delta lifecycle is designed.
- relation-index *cache* (scale-gated half) + full coverage-gap queries.
- Spec-Driver corpus importer; code↔spec sync adapters.
- Spec lifecycle transitions / approval (pairs with the absent slice-lifecycle
  transition gap, CLAUDE.md known gaps).
