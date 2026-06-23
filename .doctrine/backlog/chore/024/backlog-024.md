# CHR-024: Entity relationship design review: spec coverage, gap analysis, and redesign RFC

## Motivation

Doctrine's entity relationship model has evolved incrementally through slices
SL-046 (relation reader), SL-048 (relation writer + tier-1 migration), SL-095
(governance supersession), SL-136 (tags migration), and ad-hoc extensions. Two
ADRs govern the architecture (ADR-004 outbound-only, ADR-010 tier partition +
contract unification), and SPEC-018 captures the cross-corpus relation contract.
But the model has never been reviewed **holistically** — across the full kind set,
the write and read CLI surfaces, agent-facing guidance, and the memory corpus —
for elegance, completeness, and coverage of real-world scenarios.

The relation-vocabulary.md companion already identifies a gap (epistemic /
knowledge-record association, class 6). Several open backlog items and issues
signal friction at the margins (IMP-149 ambiguous `slices`, ISS-041 concept-map
edge invisibility, IMP-141 validate relation visibility, IMP-138 transitive
walks, IDE-015 bridge concept-map to relation graph). The knowledge record kinds
(SPEC-019) will mint new labels when they ship, and the memory relation path
uses free-form labels outside RELATION_RULES entirely.

This chore drives a structured design review across all surfaces, produces an
RFC with findings and recommendations, and will drive out a slice for any
design/code changes required.

## Cross-cutting complexity and preparatory work

This review is unusually broad — it touches the relation data model, six CLI
verb families (write/read/validate/priority/survey/export), three web graph
builders (semantic / actionability / concept map), the cordage graph
prioritisation engine, every entity kind's parser, the agent skill corpus, and
the memory store. Giving an architect agent the entire scope in one pass would
consume 200k+ tokens of onboarding before any analysis begins.

To make this tractable for agents, the review is preceded by three preparatory
subtasks that produce focused reference materials. These are not the RFC — they
are the on-ramp so that subsequent architect passes can stay within context
limits and operate on synthesised knowledge rather than raw corpus scans.

### Preparatory subtask 1: Prioritised artifact list (prose)

Produce a prioritised, annotated list of the authored artifacts relevant to the
entity relationship model. For each, note why it matters and what aspect of the
model it governs or constrains. Organised by category:

- **Specs** — SPEC-018 (primary contract), SPEC-001 (priority engine consumer),
  SPEC-005/006/016 (consolidation targets), SPEC-019 (planned extension),
  SPEC-015/014 (backlog/slice surface), SPEC-004 (shared engine), SPEC-002
  (coverage/completion).
- **ADRs** — ADR-004 (outbound-only), ADR-010 (tier partition + contract
  unification).
- **Backlog items** — ISS-041, IMP-138, IMP-141, IMP-149, IMP-053, IMP-095,
  IDE-015, IMP-134, IMP-105, ISS-046, IMP-019/020, and others touching
  relations, entity modelling, or the priority graph.
- **Skills** — close (step 5: close origin), execute, design, slice, backlog,
  audit/reconcile, code-review, record-memory, next/handover, and any that
  reference relations, `link`, `inspect`, or entity lifecycle.
- **Memories** — all ~20+ memories tagged `relations` or touching entity
  relationships, with staleness and cross-link quality notes.
- **Web graph builders** — semantic graph, actionability graph, concept map —
  how each consumes or projects relations.
- **Tests** — the exact-coverage invariant test in `relation_graph.rs`, the
  RELATION_RULES lockstep tests, golden tests for `inspect`/`show`, the
  black-box e2e tests for `link`/`unlink`.

Output: a single `.md` file in `.doctrine/state/chr-024/artifact-index.md`. (DONE)

### Preparatory subtask 2: High-relevance code map

Produce a structured map of the code files and key functions that implement the
entity relationship model. For each, note its role, the relation concepts it
implements (vocabulary, storage, validation, reading, writing), and which audit
surface(s) it feeds.

Key files to cover:
- `src/relation.rs` — RELATION_RULES, read_block, tier1_edges, validate_link,
  append_edge/remove_edge, inbound_name, legal label queries
- `src/relation_graph.rs` — outbound_for, overlay allocation, exact-coverage
  invariant tests
- `src/integrity.rs` — KINDS table
- `src/commands/relation.rs` — relation list/census CLI
- `src/commands/inspect.rs` — inspect read surface
- `src/dep_seq.rs` — needs/after typed axes
- `src/supersede.rs` / `src/commands/supersede.rs` — supersede verb
- `src/backlog.rs` — backlog relation parsing (read_block, relation_edges)
- `src/slice.rs` — slice relation parsing
- `src/governance.rs` — governance relation parsing
- `src/spec.rs`, `src/adr.rs`, `src/policy.rs`, `src/standard.rs` — per-kind
  relation parsing
- `src/review.rs` — review [target] relation
- `src/rec.rs` — rec owning_slice
- `src/knowledge.rs` — knowledge record relation seam (planned)
- `src/catalog/scan.rs` — catalog scan edge collection
- `src/priority/graph.rs` — cordage graph (priority overlay allocation)
- `src/priority/surface.rs` — survey/next/blockers/explain read surfaces
- `src/commands/link.rs`, `src/commands/unlink.rs` — write verb handlers
- `src/commands/validate.rs` — corpus validate (relation findings)
- Web graph builder sources

Output: a single `.md` file in `.doctrine/state/chr-024/code-map.md`. (DONE)

### Preparatory subtask 3: Research agent synthesis

Dispatch a research agent (focused, web-access not required — the corpus is
local) to produce concise syntheses of:

1. **The relation contract** — what RELATION_RULES says, how it drives the five
   consumers, the tier model, the write seam. 2–3 paragraphs.
2. **The entity landscape** — the KINDS set, what each kind authors and reads.
   3–4 paragraphs with a summary table.
3. **Known friction points** — the open backlog items and issues that signal
   modelling strain (ISS-041, IMP-138/141/149, etc.). 1–2 paragraphs each.
4. **The preliminary findings (F-1 through F-7)** — one paragraph each, so an
   architect can grasp the gap without reading the full chore body.
5. **Web graph builders** — how each builder consumes relations, what would
   break if the model changed.

The syntheses should be dense — a few pages total, not a full spec — so an
architect can load all five in one context window alongside the audit checklist.

Output: a single `.md` file in `.doctrine/state/chr-024/research-synthesis.md`.

### After preparatory work

Once the three outputs exist, the RFC authoring phase begins (see §8
Deliverables). The syntheses serve as the onboarding payload for each
architectural pass; the artifact index and code map prevent rediscovery and
context churn.

---

## Preliminary findings (live gaps identified during chore creation)

> These are not the RFC — they are initial observations that surfaced the moment
> the chore was created and its own relationships were interrogated. They are
> recorded here so the formal review inherits them rather than rediscovering them.

### F-1: `governed_by` excludes backlog kinds

A backlog item (ISS/IMP/CHR/RSK/IDE) cannot express `governed_by ADR-NNN` or
`governed_by POL-NNN` as a structured edge. The `governed_by` RELATION_RULES entry
has source-set `{SL, PRD, SPEC, CM, ASM, DEC, QUE, CON}` — backlog kinds are absent.

This means CHR-024 (a chore auditing ADR-004/ADR-010) has **no way to record that
it is governed by those ADRs** except prose mentions. The system under review cannot
describe its own review vehicle — direct evidence the source-set is incomplete.

Backlog items are governed by the same governance corpus as slices and specs. The
likely fix: extend `governed_by`'s source-set to include BACKLOG kinds, harmonising
with the principle that any artefact inside the governed corpus should be able to
state its governing authority.

### F-2: `related` excludes backlog kinds (except through SL/RFC)

The `related` label has two rules:
- GOV `→` SameKind (governance↔governance)
- `{SL, RFC}` `→` AnyNumbered (slice/rfc to anything)

No rule admits `BACKLOG → AnyNumbered` or `BACKLOG → BACKLOG`. A chore cannot
express `related ISS-041` or `related IMP-149` — items it directly touches — as
a structured peer edge. The only peer-association outlet for a backlog item is
`drift` (free-text, unvalidated).

### F-3: `specs` on backlog items conflates three distinct semantics

A backlog item that links to a spec via `specs` cannot distinguish:

| Reading | Meaning | Example |
|---|---|---|
| **emitted from** | This work item was *generated from* this spec's requirements/analysis | "CHR-024 was scoped from SPEC-018's open questions" |
| **implements** | This work item *realises* or *delivers* this spec | "SL-048 implements SPEC-018" |
| **peer / bears on** | This work item is *associated with* this spec without emission or implementation | "CHR-024 bears on SPEC-018 but is not implementing it" |

The current label collapses all three into one `specs` name, which renders as
"specs" on both the outbound and inbound side — losing the semantic distinction.

Options for resolution (to be explored in the RFC):
1. Mint distinct labels: `implements`, `emitted_from`, `relates_to_spec`
2. Extend `related` to BACKLOG kinds and reserve `specs` for implementation only
3. Keep `specs` but add a payload/classification facet

### F-4: CHR-024 cannot link to SPEC-018 in a semantically meaningful way

Compound of F-1/F-2/F-3: the one label CHR-024 could use (`specs`) forces a
semantic commitment — "this chore implements SPEC-018" — which is inaccurate.
CHR-024 *reviews* SPEC-018; it does not *implement* it. The review's findings
may drive a slice that implements SPEC-018 changes, but the chore itself is not
a delivery vehicle.

(See also IMP-149: ambiguous `slices` label on backlog items is a parallel
problem for the `slices` axis.)

### F-5: No `reviews` label for backlog kinds

`reviews` exists only for the RV kind (TypedVerbOnly, review→target). A backlog
item cannot say "this chore reviews SPEC-018" as a structured edge. If the model
had a peer-level `reviews` label open to backlog/slice sources, CHR-024 would
use it.

### F-6: No completeness/completion semantics on spec→slice coverage

A backlog item may require *multiple* slices to fully satisfy a spec. But the
model has no way to express partial coverage or completion conditions:

- A backlog item B links to spec S via `specs` (or to a slice via `slices`).
- Slice SL-1 implements *part* of spec S (links via `specs`,`requirements`).
- Slice SL-1 closes.
- The `/close` skill step 5 says: "Close the originating backlog item: if a
  backlog item spawned this slice, transition it too."
- But spec S needs *both* SL-1 and SL-2. Closing SL-1 prematurely closes B.

The model cannot distinguish "this spec is fully satisfied by this slice" from
"this spec is addressed by this slice but more work remains." There is no:
- Partial-satisfaction indicator (e.g., "this slice addresses X% of the spec")
- Dependency on other slices addressing the same spec ("SL-2 must also close")
- Coverage predicate on the `specs` edge ("slice SL-1 is necessary but not
  sufficient for spec S")

Currently the system relies on prose and LLM discernment to avoid premature
backlog-item closure — a brittle check the model should enforce.

(See also the coverage/reconciliation engine SPEC-002, which reconciles
individual requirements, and IMP-026 on actionability masks — neither addresses
spec-level completion from a backlog origin perspective.)

### F-7: No work-entity decomposition model (hierarchy / epic pattern)

There is no way to express that a slice or backlog item decomposes into smaller
sub-units — whether as an explicit "split this entity into children" workflow
or a long-running "epic" that contains sub-items.

**Slice decomposition.** A slice that grows too broad cannot declare sub-slices:
- `SL-010` cannot say `decomposes_to SL-011, SL-012`.
- The `supersedes` label (SL→SL) replaces an older slice wholesale — not a
  decomposition.
- The `related` label (SL→AnyNumbered) is peer association, not hierarchy.
- The `parent` label is SPEC→SPEC only (tech spec decomposition tree),
  not available for work entities.

**Backlog decomposition / epic pattern.** A backlog item cannot declare
containment or epic-parent status:
- `IMP-050` cannot say "contains IMP-051, IMP-052, IMP-053" or "is an epic of."
- The `needs`/`after` axes are sequencing, not hierarchy.
- The `slices` label (BACKLOG→SL) points at implementing slices — those slices
  are not children of the backlog item.

**What's missing:** a decomposition or containment relation for work entities —
for example `decomposes_to` (work→work), or extending `parent` semantics to
cover work entities (though `parent` is already SPEC→SPEC and would need
renaming or a new label). The absence means there is no structured way to:
- Track that a broad slice has been split into narrower slices.
- Mark a backlog item as an epic whose sub-items must all resolve before it
  can be closed.
- Display a work-item hierarchy on any read surface (`inspect`, `survey`,
  `status`).
- Validate that sub-items are resolved before the parent closes.
- Guide agents through the decomposition workflow (update parent scope,
  create children, re-link relations).

This overlaps with the completeness gap (F-6): if a slice were decomposed into
sub-slices, the parent should not close until all children have closed.

---

These six findings are the first data points for the formal audit (§6 stress-test,
§7 spec coverage). They are provisional — the RFC may confirm, refine, or reject
them after deeper analysis.

## Additional Gaps

Key gaps the context prep surfaced:

 1. No skill provides relation-label usage heuristics — agents have to
    read RELATION_RULES in Rust code to understand specs vs slices vs
    requirements vs drift vs governed_by
 2. Only 3 memories tagged relations — thin for a cross-cutting model
 3. Concept map has its own edge store, separate from the relation
    graph (ISS-041's root cause)
 4. CatalogEdgeLabel merges closed RELATION_RULES + free-form memory
    labels — a seam the review must examine


## Scope

### 1. Data model audit

- **Entity kinds and their relation rules.** Walk `integrity::KINDS` (all 17+
  numbered kinds: SL, ADR, POL, STD, PRD, SPEC, REQ, ISS, IMP, CHR, RSK, IDE,
  RV, REC, REV, RFC, CM, ASM/DEC/QUE/CON) against `RELATION_RULES` in
  `src/relation.rs`. For each kind, verify:
  - Every legal outbound label is necessary and sufficient.
  - Missing labels that real scenarios demand.
  - Correct tier classification (tier-1 `[[relation]]` vs tier-2 typed vs tier-3
    free-text).
  - `LinkPolicy` correctness (Writable / LifecycleOnly / TypedVerbOnly).
  - `TargetSpec` correctness (Kinds / SameKind / AnyNumbered / Unvalidated).
  - `inbound_name` coherence (does the derived reciprocal read naturally?).
- **Universal vs kind-specific relationships.** Distinguish:
  - Composition / lineage (`descends_from`, `parent`, `members`)
  - Authorization / governance (`governed_by`, `owning_slice`)
  - Work → artefact (`specs`, `slices`, `requirements`, `drift`, `reviews`, `revises`)
  - Peer association (`related`, `interactions`, `contextualizes`)
  - Replacement / succession (`supersedes`)
  - Free-text / external (`decision_ref`)
  - Epistemic / knowledge-record (planned: `informs`, `spawns`)
- **Storage tier boundaries.** Are tier-2/3 edges correctly excluded from
  tier-1? Should any tier-2 edges migrate down? Are there latent arity or
  payload constraints not captured?
- **Outbound-only invariant (ADR-004).** Is the `superseded_by` carve-out still
  the sole reverse edge? Are there any new reverse-field candidates?
- **Memory relation path.** Memory edges use `CatalogEdgeLabel::Raw` (free-form),
  entirely outside RELATION_RULES. Should memory have a bounded vocabulary, or
  does the free-form fork need to be reconciled?

### 2. CLI write-surface audit

- **`link`/`unlink`.** Does the verb surface all tier-1 `LinkPolicy::Writable`
  edges? Are error messages helpful when a label/kind pair is illegal? Does the
  idempotency contract (re-link is no-op, unlink absent is no-op) hold in all
  edge cases?
- **`needs`/`after`.** Are the dep/seq axes correctly separated from the tier-1
  relation model? Does the target-kind validation (must be work-like: SL or
  backlog) match the intent? Does `after --prune` compose correctly? (See
  ISS-046: needs rejects SL-prefixed targets?)
- **`supersede`.** Does the transactional co-write (`NEW.supersedes += OLD`,
  `OLD.superseded_by += NEW`, `OLD.status → superseded`) compose correctly
  across all lifecycle-aware kinds (ADR, POL, STD, and future knowledge
  records)? Refuses cross-kind? Refuses self-edge?
- **`tag`/`estimate`/`value`/`risk`.** Are classification facets (tags) and
  valuation facets correctly not relations? Check SL-136 tag migration
  completeness.
- **`link` for concept-map (`contextualizes`).** ISS-041: writable via link but
  invisible to read paths (outbound_for scan gap). Status?
- **Bespoke write verbs.** `spec req add` (members), spec `descends_from`/`parent`
  setters, review `[target]`, rec `owning_slice`. Are any of these candidates for
  tier-1 migration? Should any new typed verb exist?

### 3. CLI read-surface audit

- **`inspect`.** Shows one entity's outbound + derived inbound + unresolved
  danglers. Does it cover all entity kinds? Is the output complete and
  well-structured for agent consumption? Does it handle memory sources?
- **`relation list`/`relation census`.** Filter-and-project and label-grouped
  tallies. Do the filters (`--label`, `--target`, `--source-kind`, `--unresolved`)
  compose correctly? Does `--include-memory` surface edges whose labels fall
  outside the RELATION_RULES closed set? Is `census` useful for coverage analysis?
- **`survey`/`next`/`status`/`blockers`/`explain`.** Relation-derived priority
  views. Do they correctly aggregate blockers, soft sequences, and eligibility?
  Are records (knowledge, review, rec) correctly excluded (not Workable)? Is
  the `needs`/`after` overlay composition correct for mixed-kind dep chains?
  IMP-120 (transitive impact query) and IMP-138 (transitive walk for inspect).
- **`validate`.** Relation-specific findings (IllegalRow, dangling target,
  supersession cross-check). Does the validator cover all relation rules? Are
  findings actionable?
- **Is there a read gap?** What relation query cannot be expressed with the
  current verb set? E.g., "find all entities that relate to entity X through any
  label", "show the closure of governed_by through the ADR/POL/STD graph",
  "what specs does this ADR govern transitively".

### 4. Agent-facing skills audit

- **Skills that mention or depend on relations:**
  - `design` / `plan` / `execute` / `phase-plan` — do they guide agents to use
    `link` instead of hand-editing relation rows (mem.pattern.relation.relate-via-link...)?
  - `slice` — does scope capture properly record relationships?
  - `canon` / `consult` — do they surface relation rules when making architectural choices?
  - `spec-product` / `spec-tech` — do they guide relation vocabulary choices?
  - `close` / `audit` / `reconcile` — do they verify relation integrity at closure?
  - `backlog` — does IMP-149 (ambiguous `slices` label) create confusion?
  - `code-review` / `inquisition` — do they check relation rule compliance?
  - `record-memory` / `retrieve-memory` — memory label free-form divergence.
  - `next` / `handover` — do they surface important relation context?
- **Do skills provide a consistent mental model?** Is the tier model, outbound-only
  invariant, and verb surface explained consistently across all skills?
- **Are there skill guidance gaps?** E.g., "when should I use `specs` vs
  `slices` vs `requirements`?" — does any skill provide usage heuristics?
- **Are the MCP tools (`link`, `unlink`, `inspect`, etc.) properly documented
  with workflow guidance?** (IMP-150, IMP-151, IMP-152)

### 5. Memory corpus audit

- **Relation-related memories.** Audit the ~20+ memories touching relations for
  accuracy, currency, completeness, and cross-linking:
  - `mem.pattern.relation.relate-via-link-not-hand-authored-rows` — guidance to
    use `link` instead of hand-editing.
  - `mem.pattern.relation.authored-rows-tooling-half-wired` — correction that the
    full surface is wired.
  - `mem.pattern.link.memory-label-fork` — memory labels are free-form.
  - `mem.pattern.design.unified-read-not-unified-write` — ADR-010 design insight.
  - `mem.pattern.review.superseded-by-is-adr004-carveout` — superseded_by is
    sanctioned.
  - `mem.system.entity.numbered-kind-identity-table` — KINDS table.
  - `mem.system.spec.composition-seam` — why membership label is mobile edge data.
  - And others found through `memory find` when searching by tag:relations, tag:entity.
- **Are there knowledge gaps the agent corpus should cover?** E.g., full relation
  vocabulary reference, tier decision tree, common relation patterns.
- **Memory `retrieve` holdback.** Could any relation knowledge be suppressed by
  trust/severity gating when an agent needs it?

### 6. Scenario stress-test

Design and evaluate the model against real and imagined scenarios:

- **Real corpus scenarios** (from the existing doctrine repo):
  - A slice is governed by an ADR, specs two specs, addresses three requirements,
    supersedes an older slice.
  - An ADR is governed by a policy, related to another ADR, superseded by a third ADR.
  - A backlog issue has `slices` to two slices, `specs` to one spec, `drift` to a change.
  - A spec `descends_from` a PRD, `parent` to another spec, `consumes` from a third spec.
  - A concept-map `contextualizes` an ADR and a spec.
  - A review (RV) targets a slice; a reconciliation record (REC) owns to a slice.
  - A revision (REV) revises a spec.
  - A knowledge record (ASM/DEC/QUE/CON) bears on a backlog item and spawned a risk item.
- **Torture-test scenarios** (edge cases the model should handle):
  - A slice has no relations at all (is this a problem? should it be validated?).
  - A spec `consumes` from a spec that's been superseded (dangling chain).
  - An entity is both `governed_by` and `related` to the same target (self-loop through
    different labels — should this be allowed?).
  - A backlog item is linked to a slice via `slices`, the slice is then superseded.
    Does the backlog edge become dangling? Should it auto-follow?
  - A knowledge record spawns a work item, then the record is superseded. Should the
    spawn edge follow the supersession chain?
  - A POL supersedes a STD (cross-kind supersession — currently refused; should it be
    allowed for governance kinds?).
  - Circular `governed_by` (A governed_by B, B governed_by A) — detected? Refused?
  - Memory entity linked to a numbered entity — memory labels are free-form, but what
    about the numbered entity's perspective (derived inbound from memory)?
  - Multiple entities all `contextualized_by` the same concept map — does the concept
    map reader render all of them?
  - A spec has no `parent` (top-level spec) — correct. Same spec has no `members`
    — also correct. Is "members" optional or vet-for-non-empty?
- **Cross-cutting scenarios:**
  - Read the model through different lenses: a new contributor onboarding, a reviewer
    checking consistency, an architect designing a new entity kind, an agent deciding
    which verb to use.
  - Map all relation paths between a set of related entities and verify every edge
    has an authoring verb, a read surface, and a validation rule.
  - What happens to relation edges when the source or target entity is deleted?
    (There is no delete verb yet — IMP-062 — but the question affects design.)

### 7. Spec coverage assessment

- **SPEC-018** (Cross-corpus relation contract). Evaluate for completeness:
  - Does it cover all entity kinds that exist today?
  - Does the PHASE-06 consolidation (rewire SPEC-005, SPEC-006, SPEC-016 to
    reference this contract) need scheduling?
  - Does the `relation-vocabulary.md` companion need updating for the epistemic gap
    (knowledge records) or new labels?
  - Are the requirements (REQ-305 through REQ-309) sufficient to drive code
    verification, or do they need expansion?
  - Does it need a `Hypotheses` section about memory relations?
- **SPEC-005** (ADR entity surface) — reserved `[relationships]` seam: is it
  correctly inert, does SPEC-018 referencing need updating?
- **SPEC-016** (Governance kinds) — same question: `[relationships]` seam, relation
  wording alignment with SPEC-018.
- **SPEC-006** (Spec composition machinery) — lineage edges (`descends_from`, `parent`,
  `members`, `interactions`): are they correctly tier-2, is the spec clear?
- **SPEC-019** (Knowledge-record entity surface) — its planned relation seam
  (record→backlog-item relate + `spawns`): pre-validate the design against the
  RELATION_RULES model before shipment.
- **SPEC-001** (Graph-Derived Priority) — does the priority engine correctly consume
  all relation kinds? Does the `dep` overlay handle all kinds?
- **Do we need a separate spec for the universal relation axis?** Or is SPEC-018
  sufficient if expanded to cover universal invariants alongside kind-specific rules?

### 8. Deliverables

1. **RFC document** (`rfc/NNN/`) with:
   - Audit findings per surface (1–6 above), severity-graded.
   - Gap analysis: missing labels, surfaces, validations, guidance.
   - Stress-test outcomes: which scenarios fail or produce confusing results.
   - Recommendations: spec changes, code changes, new verbs, skill updates,
     memory updates.
   - Proposed redesign(s) for areas currently in tension.
2. **One or more slices** derived from the RFC, scoped to implement the agreed
   design/code changes.
3. **Updated SPEC-018** (or a new companion spec) covering the reconciled model.
4. **Updated agent skills and memories** to reflect the finalised model.

## Out of scope

- Implementation of any changes found (that's what the derived slice is for).
- Adding entirely new entity kinds (that's a product spec / PRD decision).
- `backlog needs`/`after`/`triggers` re-modelling (those are dep/seq axis,
  owned by SL-047 / priority engine — unless the review finds they should merge).
- Delete/archive verbs for entities (IMP-062 — separate tracked work).

## References

- SPEC-018: Cross-corpus relation contract
- SPEC-018 `relation-vocabulary.md`: Semantic classification of 17 labels
- SPEC-005: ADR entity surface (reserved `[relationships]` seam)
- SPEC-016: Governance kinds (POL/STD)
- SPEC-019: Knowledge-record entity surface (planned relation seam)
- SPEC-001: Graph-Derived Priority Engine
- ADR-004: Relations stored outbound-only; reciprocity derived
- ADR-010: Relation modelling: unify contract and write seam, keep storage bespoke
- `src/relation.rs`: RELATION_RULES table (authoritative vocabulary)
- `src/relation_graph.rs`: outbound_for reader + overlay allocation
- `src/integrity.rs`: KINDS table (every numbered kind)

Related backlog items:
- ISS-041: Concept-map contextualizes edges writable via link but invisible to read paths
- IMP-138: Relation-transitive walk for inspect
- IMP-141: doctrine validate relation visibility
- IMP-149: Ambiguous `slices` relation kind on backlog items
- IMP-053: Record↔record associative relation class (informs/bears-on) for SPEC-019
- IMP-095: Migrate record Supersedes from typed [relationships] to [[relation]]
- ISS-046: backlog needs CLI rejects SL-prefixed slice targets
- IMP-019: cordage golden_net determinism not value-correctness
- IMP-020: cordage query.rs traversal triplication
- IDE-015: Bridge concept map to relation graph
- IMP-150/151/152: MCP review tool documentation
- IMP-105: Extend lazyspec projection to new entity kinds
- IMP-134: Extend tagging to all appropriate entity types
