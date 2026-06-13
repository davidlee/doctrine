# SPEC-019: Knowledge-record entity surface

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The knowledge-record entity surface is doctrine's epistemic-and-governance capture
layer realising **PRD-010**: the durable, typed, citable home for the assumptions,
decisions, questions, and constraints that *shape* work without being work. It is a
component of the entity engine (**SPEC-004**) — four `record_kind`s riding four engine
`Kind`s over the same kind-blind materialiser, the structural sibling of the backlog
surface (**SPEC-015**). All shared mechanism (identity, the atomic claim, id
allocation, edit-preserving status transition, and the scaffold/render pipeline) lives
in the parent container and is used here unchanged; this spec carries only what is
specific to this family: the four-kind discrimination, the **per-kind lifecycle
vocabularies**, the per-kind typed `[facet]` blocks plus the shared evidence structure,
prefix→kind resolution on the read path, the outbound relation seam over the SPEC-018
contract, and the cross-kind supersession lifecycle verb.

It is **forward-intent**: no code is shipped yet, so this spec describes the planned
mechanism. The per-slice `/design` owns the concrete fileset names, module placement,
and code-impact; what is fixed here is the durable architecture and the contracts that
outlive any one change. The truth/work boundary is the spine — this family records what
*shapes* slices, the backlog, and governance; it never becomes them.

## Responsibilities

Mirrors the structured `responsibilities` list: bind the four record kinds onto the
engine, carry the per-kind lifecycle vocabularies and hide-set, hold the per-kind typed
facets and the shared evidence structure, resolve kind from the id prefix, hold the
outbound relation seam over the contract, and own the cross-kind supersession verb.

### Four kinds, one engine

A knowledge record exists in four subtypes — assumption (`ASM`), decision (`DEC`),
question (`QUE`), constraint (`CON`) — each a data-valued engine `Kind` with its own
tree under `.doctrine/knowledge/<kind>/` and its own reservation namespace, so `ASM-001`
and `DEC-001` coexist with independent counters. The subtypes share one
`record-NNN.{toml,md}` fileset and an `NNN-slug` symlink alias, diverging only in their
prefix, their lifecycle vocabulary, and the `[facet]` block the scaffold seeds. The
discrimination is the **one-entity, one-schema** rule (PRD-010 §4): `record_kind` is a
facet on one entity, never a fork of parallel per-kind schemas — the same discipline the
backlog applies to its five item kinds. The `record_kind` enum is the `clap::ValueEnum`
positional on capture and the kebab serde of the stored `record_kind` field.

The `DEC` prefix is **dual-namespaced**, deliberately (D8): the numbered decision kind is
the **2-part** form `DEC-NNN` (`DEC-003`); the entrenched **3-part** `DEC-NNN-XX` refs
already in the corpus (`DEC-005-C`, `DEC-010-06`) are *external* the external decision register
decision-log citations, not doctrine entities — they stay free-text prose and are never
renumbered. The shipped `DecisionRef` Unvalidated label and `rec.decision_ref`, which
carry those external refs today, are disambiguated inside the DEC-kind slice, not here.

```text
.doctrine/knowledge/<kind>/NNN/
  record-NNN.toml      # identity, record_kind, status, summary, tags;
                       #   then TYPED tables — [facet], [evidence], and the typed
                       #   supersession [relationships] pair;
                       #   then the tier-1 [[relation]] array rows (artefact links + spawn).
  record-NNN.md        # prose body
```

The on-disk order is load-bearing, not cosmetic: the SPEC-018 **F1 storage
invariant** requires every typed table to *precede* the `[[relation]]` array-of-tables
(a bare key after an array header binds to the last table — silent corruption). So
`[facet]`, `[evidence]`, and the typed `[relationships]` supersession pair are all
authored above the `[[relation]]` rows, and the writer appends edges only at EOF.

This is the **divergence from the backlog**: the backlog's five kinds share one closed
status vocabulary and only the risk kind carries a `[facet]`; here every kind carries
both its **own** lifecycle vocabulary and its **own** facet shape. The shared
materialiser is unchanged — the variation is data (per-kind status set, per-kind facet
seed) keyed by `record_kind`, not a parallel implementation.

### Per-kind lifecycle vocabularies

Unlike the backlog's single `Status` enum, lifecycle here is **keyed by `record_kind`**
(PRD-010 §6, the "lifecycle belongs to the kind" principle):

- **assumption** — `held → testing → validated | invalidated | obsolete`
- **decision** — `proposed → accepted | rejected | superseded`
- **question** — `open → answered | obsolete`
- **constraint** — `active → waived | superseded | retired`

Capture seeds the kind's default first state (held / proposed / open / active). Status
is hand-settable and ungated, as slices, ADRs, specs, and backlog items ship today. The
transition verb validates the target against `vocab(record_kind)` and **refuses** a
foreign-kind state rather than coercing it — the validation is kind-relative, the one
mechanism this surface adds that the backlog's closed-global-enum model does not need. A
per-kind `is_terminal` predicate (`validated`/`invalidated`/`obsolete` ·
`rejected`/`superseded` · `answered`/`obsolete` · `waived`/`superseded`/`retired`)
drives the default-list hide-set through `listing::retain`, exactly as the backlog's
`is_terminal` does — terminal records drop from the default survey but stay addressable
under `--all` or an explicit `--status`. These predicates are deliberately neither the
backlog's nor the slice's: each family owns its own terminal set.

### Typed facets and the evidence structure

Each kind seeds its own typed `[facet]` block; there is no untyped frontmatter bag
(PRD-010 NF-001):

- **assumption** — the claim, a `confidence` (low / medium / high), a `basis`
  (observation / prior-art / design-inference / external-source / operator-judgement), a
  validation plan, and the validated/invalidated by-and-on.
- **decision** — context, choice, alternatives, rationale, consequences, and the decided
  by-and-on.
- **question** — the question, why it matters, the answer, and the answered by-and-on.
- **constraint** — the statement, its `source` (canon / adr / external / technical /
  legal / compatibility / operator), what it applies to, and the waiver reason and
  waived by-and-on.

`confidence` is an **assumption-only** facet, not a common field (PRD-010 OQ-004) — the
common schema is identity, summary, and tags only. The closed enums (`confidence`,
`basis`, constraint `source`) ride the same `"" -> None` optional seam the backlog's
`Resolution`/risk axes use: a tolerant raw parse reads them as strings, a separate
`validate` pass maps empty to absent and parses any non-empty token to its variant,
erroring on an unknown one. This is the parent container's three-layer parse model,
specialised for seeded-empty optionals.

Evidence is a single **shared** minimal support structure across all four kinds — a
typed `[evidence]` table of `supports` / `contradicts` / `notes` citations (PRD-010
NF-001). It is **not** its own entity kind and **not** a free-form blob; in v1 it is a
minimal citation structure, never queryable graph machinery (PRD-010 §2 out-of-scope).

### Prefix→kind resolution on the read path

Capture takes `record_kind` explicitly and derives the prefix from the engine `Kind`
(the single source), as the backlog does. The **read** path inverts this: `show`,
transition, relate, and supersede name a record by id and resolve `record_kind` from the
id **prefix** (`ASM`/`DEC`/`QUE`/`CON` → the kind), so one verb set dispatches across all
four prefixes and selects the right lifecycle vocabulary and facet shape. Identity — the
kind prefix plus number — is permanent; the slug is never authoritative; and
`record_kind` is fixed at capture and never silently changes (PRD-010 NF-003). This
prefix→kind resolver is the half PRD-010 §2 explicitly handed to the technical spec.

### The outbound relation seam

A record is only interesting *in relation to other entities*, so the relation seam is
the load-bearing surface, not an afterthought. It follows the **cross-corpus relation
contract** — the model SPEC-018 specifies and SL-046/SL-048 shipped — never a bespoke
record relation store. This spec does not re-tell that model; it names the concrete
extension points the family must ride and the one gap it opens.

The contract is realised by two cooperating layers this surface plugs into:

- **The authored-edge layer (SL-048).** `RELATION_RULES` (`src/relation.rs`) is the
  code-authoritative legal-set table keyed by `(source ∈ sources, label)`, each rule
  fixing a `TargetSpec`, a `Tier`, and a `LinkPolicy`; the uniform `link`/`unlink`
  verb over `append_edge`/`remove_edge` is the writer, gated by that table.
- **The reader layer (SL-046).** `relation_graph::outbound_for` dispatches per
  canonical prefix to the kind's `relation_edges` accessor and emits
  `RelationEdge{label,target}`; inbound is **derived** from `in_edges`, never stored
  (ADR-004). An **exact-coverage invariant test** holds the two in lockstep — per
  source kind, the reader's emitted labels equal the table's labels.

Plugging the four record kinds in therefore requires, concretely:

1. **`integrity::KINDS` rows** for ASM/DEC/QUE/CON (kind constants + each kind's
   stateful status set) — the corpus-wide id table the contract and graph both scan.
2. **A `RECORD` source-group and rules in `RELATION_RULES`** — the records' outbound
   labels. Most **reuse existing** labels by joining the `RECORD` source-group to their
   `sources` set, the target sets already matching: `specs` (→ PRD/SPEC), `slices`
   (→ SLICE), `requirements` (→ REQ), `drift`, and `governed_by` (→ the governance set —
   the record→ADR axis). **Two axes have no existing label and must be minted** (the
   reuse option is foreclosed — see the verdict below): a record→backlog-item *relate*
   label targeting the five backlog kinds (so record→risk is just this label aimed at
   `RSK`) and a distinct `spawns` origin label (e.g. `ASM-001 → RSK-004`,
   `DEC-003 → SL-020`). All are `Tier::One` (`[[relation]]`) and `Writable`; the inbound
   origin on the spawned/linked artefact is derived, never authored on it.
3. **An `outbound_for` dispatch arm** (or a shared record accessor, mirroring the
   five-backlog-kinds-one-accessor pattern) — `outbound_for` `debug_assert!`s that
   every `KINDS` prefix is routed, so a new kind with no arm is a loud invariant
   breach, not a silent empty.
4. **Extending the exact-coverage invariant** to the record source kinds, so the
   reader and the table cannot drift.

**The closed-vocabulary verdict (pinned, not deferred):** `RelationLabel` is a *closed*
enum, so PRD-010's link list resolves into exactly three classes, and fixing that
classification is the one decision a tech spec owes its slice:

1. **Source-set extensions** — `specs`/`slices`/`requirements`/`drift`/`governed_by`
   already exist with matching target sets; the record kinds simply join their `sources`.
   No new label. record→ADR rides `governed_by` (the governance sense); **no separate
   peer-relate to an ADR is in v1** — minting one is deferred until a concrete
   non-governance need surfaces (avoid over-minting).
2. **New variants** — a record→backlog-item *relate* label and a `spawns` origin label.
   Nothing existing targets the backlog kinds (`Slices` is `BACKLOG`-sourced and
   `SLICE`-targeted, and a record is in neither set), so **spawn cannot reuse
   `slices`/backlog labels** — the option the prior draft left open is closed. Each
   minted variant carries a wire name, an `inbound_name`, and a rule row.
3. **Reused as-is** — `Supersedes`, via a new RECORD-sourced `LifecycleOnly` rule row
   (next section).

The *rule rows themselves* live in `RELATION_RULES`, never transcribed here (storage
rule); what this spec fixes is the **class of each link**. The seam is always present,
even empty, and relating or spawning never mutates the linked artefact's lifecycle.

### Cross-kind supersession

Supersession is a **replacement edge distinct from a relation** (PRD-010 §6), admitted
only when the successor becomes the authoritative continuation of the predecessor. It is
the `supersedes` / `superseded_by` pair — the ADR-004 §5 reverse carve-out, co-written on
both records, sanctioned because the supersession already moves the predecessor to a
terminal status and rewrites its file, so the reverse edge adds zero marginal coupling.
In `RELATION_RULES` terms this is the `Supersedes` label at `LinkPolicy::LifecycleOnly`
— never plain-`link` — added as a **new RECORD-sourced rule row** whose `TargetSpec` is
the four record kinds (cross-kind *within the family*, unlike the governance row's
`SameKind`). Like governance supersession (SPEC-018 OD-3), it is
**storage-excluded from the tier-1 `[[relation]]` migration**: the pair stays a typed
`[relationships]` block (mirroring SPEC-005's ADR seam) because the sanctioned reverse
`superseded_by` is structurally un-authorable as a `[[relation]]` row — the table admits
no inverse label. It is owned by a **transactional supersede verb** — the cross-kind
lifecycle axis the corpus has been deferring as **IMP-006** (cited by SPEC-005 and
SPEC-018 as the unbuilt owning verb for governance supersession); this surface is its
first real consumer. The verb co-writes both edges atomically, moves the predecessor to
a terminal status valid for **its own** kind without changing that kind, and admits a
`record_kind` crossing only along the **§6 allowed matrix** when the successor is the
authoritative continuation — refusing a reopening direction (`constraint → assumption`,
`decision → question`) as a relation, not a supersession. The matrix is a
predecessor→successor relation `TargetSpec` cannot express (it constrains only the legal
target *kind set*), so its enforcement lives in the verb, not the contract table.

### Priority-engine posture: never actionable, meant to gate (SL-047)

Records ride the relation graph (SL-046) but stand outside the **actionable channel**
(SL-047 / SPEC-001): *truth is not work*, so a record is never something the worklist
tells you to *do*. That is the durable invariant — **no record state is ever
`Workable`**, so records are never `eligible`, never in `survey`/`next`, and carry zero
work-lineage consequence (record→artefact labels stay out of `counts_toward_consequence`
— a record *shaping* an artefact is not the artefact *depending on it for work*). Admitting
the four kinds to `integrity::KINDS` forces **four full `priority::partition` entries** —
one per kind, each `workable: &[]` with *every* status of that kind in `terminal` (the
invariant requires `workable ∪ terminal` to cover the whole vocabulary, else
`Unrecognised`), plus the four VT-1 drift canaries binding each kind's `*_STATUSES` const.
This is **not** the status-less REC path: REC carries no status and rides `status_class`'s
`None → Terminal` fast path with no table entry; records are status-ful and *cannot*, so
the interim is an explicit all-`Terminal` partition — a **positive declaration, not an
omission**.

But *never actionable* is not *never relevant to the graph*. Records are meant to **gate**
work — the load-bearing expressions are blocking ones:

- an `open` `QUE` gates the **design of a slice** (a spike answers it);
- a `held`/`testing` `ASM` gates an `IDE` until verified;
- an `active` `CON` gates a `REQ` or `SL` while it holds;
- a `proposed` `DEC` gates an `ISS` until made.

In each, the record gates a dependent but is itself never the work — and the dependent
unblocks when the record settles to a terminal state. The **current** binary partition
cannot express this: `blocked_by` reserves blocking to non-`Terminal` nodes, which are
necessarily also `eligible`, so there is no "blocks but is not itself work" class.
Decoupling the two — a third `Gating` status-class (unsettled record states), plus a
record→item gating edge allocated into the `dep` overlay — is a priority-engine
(SPEC-001/PRD-011) change tracked as **IMP-047**. The settle→unblock semantics fall out:
a record at a terminal status leaves the `Gating` class and stops gating.

**Until IMP-047 lands, records are parked in an explicit all-`Terminal` partition**
(status-ful, not REC's status-less path — neither eligible nor blocking), and a belief
that gates work does so *indirectly*, through a
**spawned backlog item** (PRD-010 §6): that item is workable, joins the `dep` overlay,
and blocks the dependent, while the record links to it. The two paths compose under the
target model — the spike (work) answers the `QUE` *and* the `QUE` gates the slice
directly. **Risks need no special handling either way** — a risk is a backlog item, not a
`knowledge_record` (PRD-010 §2), already in the graph via the backlog's cordage exposure
(SPEC-015).

### Command surface

The family is fronted by one command namespace, `doctrine knowledge`, riding the uniform
`<kind> <verb>` grammar and kind-blind listing spine **SPEC-013** owns — restated nowhere
here. The verb set is the shared one plus the family's lifecycle verbs:

- **`knowledge new <record_kind> [title]`** — capture; `record_kind` is the
  `clap::ValueEnum` positional (assumption/decision/question/constraint), reserving an id
  in that kind's namespace and seeding its default state, typed `[facet]`, and empty
  evidence/relation seams (mirrors `backlog new <item_kind>`).
- **`knowledge show <ID>`** — reassemble identity, kind, state, summary, the kind
  `[facet]`, evidence, and relations; kind auto-detected from the id prefix.
- **`knowledge list`** — survey, carrying the mandatory `CommonListArgs` spine
  (`--filter`/`-f`, `-r`, `-i`, `--status`/`-s`, `--tag`/`-t`, `--all`/`-a`, `--format`,
  `--json`, `--columns`), the status known-set check, the canonical-id form, and the
  JSON/columns model — all SPEC-013's substrate. Filters AND together; terminal records
  hidden by default (`--all` / explicit `--status` reveal). The `--status` known-set is
  **kind-relative** (the union per kind), the one place this surface's `list` diverges
  from a closed-enum kind.
- **`knowledge status <ID> <state>`** — the lifecycle transition (the shared transition
  seam), validating `<state>` against the record's own kind vocabulary and refusing a
  foreign-kind state.

Relate and supersede do not get bespoke kind verbs: **relate** rides the uniform
`link`/`unlink` verb — **already shipped and wired** (SL-046/SL-048), so FR-005 is
v1-deliverable once the record kinds carry `RELATION_RULES` rows and a `relation_edges`
reader; it is blocked on minting the new labels, not on an unbuilt verb. **Supersede** is
the transactional IMP-006 verb (cross-kind, lifecycle-gated) — genuinely unbuilt, so
FR-006 is IMP-006-gated and lands after it. Both cited above, neither restated. The
surface is pinned the same way every kind's is: the SPEC-013 parse-conformance matrix
plus per-verb black-box goldens.

## Concerns

- **Kind-relative status validation.** The transition vocabulary is selected by
  `record_kind`; the known-set per kind must mirror that kind's status enum exactly (a
  per-kind analogue of the backlog's `backlog_statuses_matches_the_variants` guard). The
  failure mode is a kind's known-set drifting from its enum, or a transition validated
  against the wrong kind's vocabulary after a prefix→kind misresolution.
- **Facet-enum drift.** The closed facet enums (`confidence`, `basis`, constraint
  `source`) each need a known-set guard mirroring their variant set — the facet analogue
  of the status canary above — or a renamed/added variant silently slips the `validate`
  pass.
- **Disjointness with the backlog.** No `record_kind` may be admitted as a
  `backlog_item.item_kind` and no `item_kind` as a `record_kind` (PRD-010 §4); the two
  families are disjoint by the work-intake membership test. The two prefix sets and two
  `KINDS` partitions must not collide.
- **Supersession atomicity.** The two-record co-write plus the predecessor's terminal
  transition must be one transactional unit, or a crash leaves a half-written lineage
  (forward edge without reverse, or reverse without the terminal status). This is why
  supersession is a bespoke verb, not two plain `link`s.
- **Matrix enforcement is semantic, not structural.** The §6 matrix bounds *which*
  cross-kind crossings are valid; the verb must enforce it, since the relation contract's
  kind-legality check alone cannot distinguish authoritative continuation from mere
  influence.
- **Relation lockstep across SL-046/SL-048.** Admitting the record kinds touches four
  coupled sites — `integrity::KINDS`, `RELATION_RULES`, the `outbound_for` dispatch, and
  the exact-coverage invariant test. They must move together: a `KINDS` row with no
  `outbound_for` arm trips the routing `debug_assert!`, and a reader/table label
  divergence trips the coverage invariant. The `outbound_for` guard is a `debug_assert!`
  only — in release an unrouted prefix falls through to a silent empty edge list, so the
  lockstep is test/debug-time, not a compile error. The blast radius also includes two
  **ordered** goldens — `kinds_table_covers_the_numbered_kinds` (the prefix vector is
  pinned in order) and `sources_match_shipped_accessors` — both of which the four new
  kinds edit. The storage-ordering F1 invariant (typed tables before `[[relation]]`
  arrays) is the other on-disk hazard.
- **Closed-vocabulary coverage gap.** `RelationLabel` is closed; PRD-010's link list
  resolves into source-set extensions plus two minted variants (the record→backlog-item
  relate label and `spawns`) — the verdict is pinned in D6, not deferred. Under-minting
  silently drops a legal link; over-minting leaves an un-routed label tripping the
  coverage invariant.
- **Forced partition entry.** Adding the record kinds to `integrity::KINDS` obliges
  **four** `priority::partition` entries (one per kind), each `workable: &[]` /
  all-`Terminal`, plus four VT-1 drift canaries — *not* the status-less REC path. No
  record state is ever `Workable`; IMP-047 later splits the unsettled states into the
  `Gating` class. The hazard is forgetting an entry (→ `Unrecognised`), or mis-classing a
  live state `Workable` and leaking records into `next` as fake work.
- **Behaviour preservation.** This family rides the shared entity scaffold *and* the
  shipped relation machinery; introducing it must leave the existing slice / ADR / spec /
  backlog / memory suites — and the relation-contract suites — green unchanged (PRD-010
  NF-002).

## Hypotheses

- **One entity discriminated by `record_kind` beats four schemas.** The four kinds share
  enough structure (fileset, identity, evidence, relations, reassembly) that one
  kind-blind materialiser serving all four — diverging only by prefix, lifecycle
  vocabulary, and facet seed — is preferred over four parallel implementations, exactly
  as the backlog's single-entity discipline (SPEC-015) proved for its five kinds.
- **Per-kind lifecycle is data, not a new engine.** The divergence from the backlog —
  four status vocabularies instead of one — is a `record_kind`-keyed lookup over the same
  edit-preserving transition seam, not a second transition mechanism. The engine stays
  kind-blind; the kind table carries the per-kind status set, as `integrity::KINDS`
  already carries a stateful status set per kind.
- **Supersession reuses IMP-006, not a bespoke fork.** The transactional supersede verb
  is the same cross-kind lifecycle axis governance supersession needs; building it for
  this family and for ADR/POL/STD as one verb avoids the parallel implementation SPEC-018
  OD-3 warns against.

## Decisions

- **D1 — `parent = SPEC-004`, `descends_from = PRD-010`; the thin-not-anaemic component
  shape.** Identity, claim, id allocation, edit-preserving transition, and the
  scaffold/render pipeline are the parent container's and are restated nowhere here; this
  component owns only the four-kind discrimination, the per-kind lifecycles and facets,
  the evidence structure, prefix→kind resolution, and the supersession verb. It is the
  structural sibling of SPEC-005 (ADR) and SPEC-015 (backlog).
- **D2 — lifecycle and facets are `record_kind`-keyed, not global.** Each kind owns its
  status vocabulary, its `is_terminal` set, and its `[facet]` shape; no single shared
  status set is imposed and no kind is forced into another's states. This is the
  deliberate divergence from the backlog's one-closed-enum model, demanded by PRD-010's
  "lifecycle belongs to the kind" principle.
- **D3 — relations follow the SPEC-018 contract; supersession is `LifecycleOnly`.** The
  outbound relation seam is authored over `RELATION_RULES`, outbound-only with reverse
  derived (ADR-004); the `supersedes`/`superseded_by` pair stays the sanctioned typed
  carve-out, owned by the transactional supersede verb (IMP-006), never plain-`link`. The
  cross-corpus model, vocabulary, and validation policy live in SPEC-018, not here.
- **D4 — capture takes the kind, the read path resolves it from the prefix.** Capture is
  kind-explicit (the prefix derives from the engine `Kind`); `show`/transition/relate/
  supersede are kind-implicit, resolving `record_kind` from the id prefix so one verb set
  serves all four kinds. `record_kind` is fixed at capture and identity is permanent.
- **D5 — evidence is a shared minimal typed structure, not a kind.** All four kinds share
  one `[evidence]` table of typed `supports`/`contradicts`/`notes` citations; it is never
  its own entity kind and never a free-form blob, and graph/search machinery over it is
  out of v1 (PRD-010 §2).
- **D6 — the relation seam extends the SL-046/SL-048 machinery, never forks it; the
  label classes are pinned.** Record relations are realised by adding `integrity::KINDS`
  rows, a `RECORD` source-group and rules in `RELATION_RULES`, an `outbound_for` dispatch
  arm, and extending the exact-coverage invariant. The closed link list resolves into
  three pinned classes (§ relation seam): **source-set extensions**
  (`specs`/`slices`/`requirements`/`drift`/`governed_by`, the record→ADR axis riding
  `governed_by` — no separate peer-relate in v1), **two minted variants** (a
  record→backlog-item relate label + `spawns`, reuse foreclosed by the table), and
  **`Supersedes` reused** via a new RECORD `LifecycleOnly` rule row. No bespoke per-record
  store, no second reader: one contract, one graph. The relate path rides the
  **already-shipped** `link`/`unlink` verb — FR-005 is v1-deliverable, label-blocked not
  verb-blocked; the `LifecycleOnly` supersession pair is owned by the **unbuilt** IMP-006
  verb, so FR-006 is IMP-006-gated.
- **D7 — records are never actionable, but they gate work; gating needs IMP-047.** The
  durable invariant: no record state is ever `Workable`, so a record is never `eligible`,
  never in `survey`/`next`, and carries zero work-lineage consequence — *truth is not
  work*, enforced at the rank layer. **But records are meant to *gate* dependents** — an
  `open` `QUE` blocking a slice's design, an `active` `CON` blocking a `REQ`, a `held`
  `ASM` blocking an `IDE`, a `proposed` `DEC` blocking an `ISS` — which the *current*
  binary `Workable|Terminal` partition cannot express (blocking is reserved to
  non-`Terminal` nodes, which are necessarily also `eligible`). Decoupling blocking from
  eligibility — a third `Gating` status-class plus a record→item gating edge into the
  `dep` overlay — is **IMP-047**, a priority-engine (SPEC-001/PRD-011) change. Until it
  lands, records are parked in an explicit all-`Terminal` partition (status-ful, *not*
  REC's status-less `None → Terminal` path) and a belief gates work only indirectly,
  through a *spawned backlog item* (PRD-010 §6). The target is direct gating; the interim
  is inert. The partition invariant forces this to be a *declaration*, not a silent skip
  of SL-047.
- **D8 — `DEC` is dual-namespaced; the decision kind takes the 2-part form, external
  citations keep the 3-part.** The numbered decision kind is `DEC-NNN` (2-part); the
  entrenched `DEC-NNN-XX` (3-part) refs in the corpus are *external* the external decision register
  decision-log citations, not doctrine entities, and stay free-text prose — never
  renumbered (provenance). The shipped `DecisionRef` Unvalidated label and
  `rec.decision_ref`, which carry those external refs today, are disambiguated inside the
  DEC-kind slice (the stale "DEC is not a numbered kind" comment, the test fixtures, the
  `--decision` example), not here. Doctrine's own doc-local decisions remain the bare
  `D1` form (glossary), never `DEC-`.

## Open Questions

- OQ-1 — the **memory-interaction seam** is deferred to v2 and out of this spec's
  mechanism. PRD-010 §3 fixes the record↔memory boundary *policy* (govern → record;
  recall → memory; promote by linking, never mutate), and PRD-010 OQ-006 (promotion path,
  memory → record) and OQ-007 (recall bridge, `memory retrieve` surfacing record refs)
  hold the open design. Both need the memory↔record link to cross the named-identity /
  numbered-kind divide — memory is `mem_<uid>`, while the SPEC-018 contract targets
  numbered kinds in `integrity::KINDS` — so the seam is cross-spec (SPEC-007 + SPEC-018 +
  this surface) and is not specified here. This spec assumes only that a record is a
  *citable* target; how memory cites or is promoted into one is owned elsewhere when v2
  opens it.
- OQ-2 — should a record ever be a **first-class blocker** in the priority graph?
  **Resolved: yes** — records are meant to gate work directly (D7, the priority-engine
  posture). The blocker is mechanism, not intent: SL-047's status partition is **binary**
  (`workable | terminal`), and blocking is reserved to non-terminal nodes — which are also
  necessarily `eligible` and thus `next`-actionable, so there is no "blocks but is not
  itself work" class. The fix — a third `Gating` status-class decoupling blocking from
  eligibility, plus a record→item gating edge into the `dep` overlay — is a priority-engine
  (SPEC-001/PRD-011) change tracked as **IMP-047**, out of this surface's scope. Until it
  lands records are `Terminal`-inert and gate only via a spawned backlog proxy (PRD-010
  §6). This OQ now tracks IMP-047, not an open design question.
