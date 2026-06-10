# SL-021 — Backfill Doctrine technical-spec corpus · design

<!-- Reference forms: entity ids padded (SPEC-001, PRD-008, REQ-082); doc-local
     refs bare (D1, OQ-1). See .doctrine/glossary.md § reference forms. -->

## 1. Frame

The tech-spec corpus holds two forward-intent specs (SPEC-001 active, SPEC-002
draft — both describe unbuilt engines); the shipped architecture itself is
uncaptured. SL-022 landed the full relational spine PRD-012 v1 requires:
`descends_from` (tech→PRD, single-valued, validated), `parent` (single-parent
acyclic decomposition), `c4_level`, `[[source]]` anchors, `interactions.toml`
peers — all hand-edited TOML with `spec validate` as the integrity gate. This
slice authors the retrospective corpus — the tech-side analog of SL-019 —
dogfooding that spine on Doctrine's own shipped architecture.

Governing inputs: PRD-012 (the spec family's product intent — both draft PRDs
are explicitly leant on, per user), PRD-013 (two-tier truth: authored status vs
observed coverage; no derivation), ADR-004 (outbound-only relations), SL-019
design + inquisition (prior art for the taxonomy → exemplar → skill → fan-out →
validate shape).

Locked decisions (design Q&A):

- **D1 Taxonomy topology** — whole-system **context** root ("Doctrine", the
  tool) + mechanism containers + thin capability components (§2). **Re-locked
  per F1-A (§11): the root is a whole-system synthesis, not the entity engine;
  every container parents to it, entity-engine content descends to its own
  container.**
- **D2 Descent rule: capability-complete** — every **shipped-mechanism** PRD
  reachable via ≥1 `descends_from` (F2: not "every active PRD" — unbuilt
  mechanism is exempt, named in §7); shared substrate carries thin
  per-capability component specs rather than restating mechanism (§2).
- **D3 Exemplar: vertical path** — root → container → component trio locks all
  three shapes before fan-out (§3).
- **D4 Requirement status: `pending`** — SL-019 posture; no audit has
  established coverage and no explicit reconciliation act has occurred
  (PRD-013); hand-editing to verified would assert unassembled evidence.
- **D5 Skill reconciliation** — exemplar-driven SKILL.md rework; template
  untouched (§4).
- Defaults carried: specs land `draft`, flip `active` at close; fan-out
  mechanism decided at `/phase-plan`.

## 2. Taxonomy (D1/D2) — candidate tree

Method per SL-019: the roster below is the design's *candidate shape*; the
confirmed roster + per-spec source map are **disposable scaffolding** (phase
sheet / handover), never a committed artifact. PHASE-01 owns the final
boundary calls.

```
context   Doctrine (the tool)              synthesis of the whole system; no parent, no descent
├─ container  Entity engine                 ← doc/entity-model.md seed; descent: — (shared substrate)
│  ├─ component  Slice entity surface       ← PRD-001 (lifecycle FSM, phases, plans)
│  ├─ component  ADR entity surface         ← PRD-008
│  ├─ component  Backlog entity surface     ← PRD-009
│  └─ component  Governance kinds (POL/STD) descent: — (no owning PRD; SL-030/SL-033, ADR-009)
├─ container  Spec composition machinery    ← PRD-002 (REQ peers, members, reassembly, validate)
│  └─ component  Tech-spec spine            ← PRD-012 (descent/parent/c4/anchors — SL-022)
├─ container  Memory engine                 ← PRD-004 (doc/memory-spec.md, richest source)
├─ container  Reservation & id allocation   ← PRD-005
├─ container  Install & distribution        ← PRD-006 (rust-embed, manifest, templates)
├─ container  Skills distribution           ← PRD-003 (plugins/ source-of-truth, symlink tree)
├─ container  Boot snapshot                 ← PRD-007 (src/boot.rs, governance projection)
├─ container  Dispatch & worktree           descent: — (ADR-006 governs; no PRD)
├─ container  Priority engine               ← SPEC-001 (pre-exists; parent ← root, PHASE-05 retrofit)
└─ container  Reconciliation                ← SPEC-002 (pre-exists; parent ← root, PHASE-05 retrofit)
   (CLI surface — SL-025 uniform contract: candidate container, descent —)
```

The **root is a whole-system context spec** ("Doctrine", the tool) — a synthesis
with no parent and no descent (REQ-085 admits anchor-free context specs). It is
**not** the entity engine: `doc/entity-model.md`'s durable content seeds the
**entity-engine container** (one container among peers), not the root. Every
container parents to the root, so the corpus is one legible tree without
asserting false containment — the root *contains* its containers by C4
decomposition, it does not *peer* with them (PRD-012 §3).

- **Unit = architectural capability** (the *how*) — not 1:1 with `doc/*` files
  nor with PRDs, and explicitly not a `src/` directory map (repo structure is
  C4 *code* altitude; hand-authoring stops at container/component, PRD-012
  OQ-5).
- **Capability-complete descent (D2, refined post-adversarial):** every PRD
  **whose mechanism ships** gets ≥1 descending spec. Where one mechanism
  realises many PRDs (the entity engine), thin per-capability **component**
  specs carry the descent, each a child of the engine container, pointing at
  the parent for shared mechanism — never restating it. `descends_from` is
  single-valued, so this is the only non-arbitrary wiring. **Exemptions are
  named, not silent:** PRD-010 is active but its `knowledge_record` family is
  unbuilt — no retrospective spec exists to author; forward-intent authoring
  is out of scope. The coverage audit (§7) lists exemptions explicitly.
- **POL/STD have no owning PRD** (PRD-007 orients, does not police; the kinds
  shipped via SL-030/SL-033 under ADR-009) — their component's descent stays
  empty, like dispatch.
- **PRD-011 / PRD-013 already covered** by SPEC-001 / SPEC-002 (forward
  intent satisfies reachability). They become **containers under the root**;
  retrofit `parent` onto both in PHASE-05 (single-field edits, F5) so the
  corpus forms one tree, not a forest.
- **Excluded:** drift ledger (unbuilt, no PRD — backfill covers shipped
  mechanism only).
- ~13–15 new specs (~15–17 corpus incl. SPEC-001/002) — the whole-system root
  is net-new on top of the entity-engine container (F1-A).

## 3. Exemplar trio (D3)

Three specs authored end-to-end, one per shape, before any fan-out:

1. **Whole-system context spec** — "Doctrine" (the tool). `c4_level =
   "context"`, no parent, no descent. A **synthesis** of the whole system —
   what Doctrine *is* and how its containers compose — authored anchor-free
   (REQ-085 admits anchor-free context specs), **not** a lift of any one
   subsystem doc. It does **not** carry entity-engine mechanism; that descends
   to the container (#2). Requirements: system-wide NF-flavoured invariants
   (storage rule as a cross-cutting principle, outbound-only relations,
   pure/imperative split). The hardest shape to keep at altitude — the bar is
   "names the parts and their composition," never restating any container's how.
2. **Entity-engine container** — parent = root, descent empty,
   `c4_level = "container"`. Carries `doc/entity-model.md`'s durable content
   (the storage rule's entity-engine realisation, entity-vs-facet taxonomy,
   identity/reference model, family-specific status vocab, runtime-state
   boundary, three-layer Rust model Raw → Entity → Registry) under the F4
   interim-authority rule (§9), plus `src/entity.rs` + registry/integrity seams
   (`integrity::KINDS`), `doc/relation-index.md`,
   mem.system.engine.identity-claim-seam,
   mem.pattern.entity.kind-is-data-not-trait. **Altitude filter (lower-stakes
   now it guards a container, not the root):** lift shipped reality only —
   entity-model.md's direction/migration/adjudication content (roadmap,
   importer stance, spec-driver critique) is *not* shipped how and stays in
   `doc/*`. The commonest fan-out shape.
3. **Thin capability component** — "ADR entity surface", parent = engine
   container, `descends_from = "PRD-008"`, `c4_level = "component"`. Anchors:
   the ADR kind module (path verified at authoring). Locks the thin shape:
   kind-specific config, statuses, render contracts only; shared mechanism
   cited via parent + `interactions`, never restated.

Per spec, three coordinated writes (SL-019 D-1 carries over): identity TOML
incl. hand-edited spine fields + template-shaped md body (Overview /
Responsibilities / Concerns / Hypotheses / Decisions) + `spec req add`
entities — `pending` (D4), present-tense statements of shipped obligations.

Gates before fan-out: `spec validate` green; `spec show` reassembles all
three; **user reviews and accepts the trio as the bar**. SPEC-001 remains the
prose-quality reference; the trio additionally locks the *retrospective*
altitude — mechanism-as-shipped, no current-vs-target language (the inverted
SL-019 skew risk).

## 4. Skill reconciliation (D5)

Canonical source `plugins/doctrine/skills/spec-tech/SKILL.md` (29 lines)
predates SL-022 — knows `c4_level`/`sources`, silent on the relational spine.
Exemplar-driven rework after the trio locks, before fan-out (SL-019 PHASE-03
pattern):

- **Add the spine:** `descends_from` (tech→PRD, single-valued, validated),
  `parent` (single-parent acyclic containment), and the containment-vs-peer
  rule — `parent` is never an `interactions` edge and vice versa (PRD-012
  principle).
- **C4 guidance:** hand-authored specs normally stop at container/component;
  code-level is exceptional (PRD-012 OQ-5 posture).
- **Authoring reality:** spine fields are hand-edited TOML — no CLI flag
  exists (`spec new` takes subtype/title/slug only); `spec validate` is the
  integrity gate. Same three-write model as product.
- **Posture:** both retrospective (shipped *how*) and forward-intent
  (SPEC-001/002 style) specs are legal per PRD-013, provided *planned* stays
  distinguishable from *verified*; requirements are REQ entities, `pending`,
  no coverage tables, no status derivation.
- **Point at the exemplar trio** as the canonical three shapes.

Distribution footgun: after editing `plugins/`, refresh = `doctrine skills
install` + `touch src/skills.rs` + rebuild
(mem.pattern.distribution.skill-refresh-command). Template untouched →
template re-embed ritual N/A. Binary-path trap still applies to all authoring:
use `cargo run --` or the `cargo metadata`-resolved binary, never
`./target/debug` (stale in the jail —
mem.pattern.build.jail-target-redirect).

## 5. Fan-out

Remaining ~11–13 specs authored from exemplar trio + reconciled skill +
per-spec source map (map lives in the gitignored phase sheet). Authoring
order **top-down**: a child's `parent` must resolve, so containers land
before their components (or ids are reserved first) — fan-out runs in waves
(containers, then components), encoded in the phase sheet. Collision safety carries
from SL-019: each spec its own entity tree; `REQ-NNN` via the atomic mkdir
claim with bounded retries → cap concurrent authors. Mechanism (Workflow
fan-out vs serial `/execute`) decided at `/phase-plan`, not design-locked.

## 6. Edges

- `descends_from` per the §2 tree; the root + shared-substrate containers
  stay empty.
- `parent` wiring per tree; **retrofit SPEC-001/SPEC-002 `parent` = the
  whole-system root** so the corpus is one tree (mechanical single-field add,
  F5).
- `interactions.toml` only where weighty — peer `uses`/`calls` distinct from
  containment (e.g. memory engine *uses* reservation for id allocation;
  install *uses* skills distribution). No edge for what containment already
  says.

## 7. Validation / closure

- `doctrine spec validate` clean corpus-wide — exercising the SL-022 checks
  for real: parent resolution + acyclicity, descent targets resolve to
  product specs, no dangling interaction FKs, no orphan REQs, no duplicate
  labels.
- `spec show` reassembles every spec; spine fields render.
- **Capability-coverage audit:** every shipped-mechanism PRD reachable via ≥1
  `descends_from` (D2) — checked by registry scan/grep at close; exemptions
  (PRD-010, mechanism unbuilt) listed explicitly, never silently skipped.
- Verified against code during design (adversarial pass): parent-cycle,
  self-parent, and second-parent checks ship (src/registry.rs §integrity,
  SL-022 sweep tests); `validate` tolerates draft descent targets (baseline
  corpus clean with SPEC-002 → PRD-013 today).
- Specs flip `draft` → `active` at close
  (mem.pattern.entity.edit-preserving-status-transition).
- `just check` green; no taxonomy/source-map artifact committed (grep the
  diff at PHASE-05, SL-019 CHARGE VII pattern).

## 8. Phase shape (provisional — for `/plan`)

1. **PHASE-01 Taxonomy** — confirm tree + per-spec source map (gitignored
   scaffolding); settle boundary calls (slices: component vs container;
   PRD-010 shipped subset; CLI-surface container in/out).
2. **PHASE-02 Exemplar trio** — whole-system root + engine container + ADR component;
   `validate`/`show` green; user accepts bar.
3. **PHASE-03 Skill rework** — exemplar-driven SKILL.md; `skills install` +
   `touch src/skills.rs` refresh.
4. **PHASE-04 Fan-out** — remaining specs, top-down, capped width.
5. **PHASE-05 Edges + validate + coverage audit** — interactions,
   SPEC-001/002 parent retrofit, corpus-wide gates, status flips.

## 9. Risks

- **Altitude drift (inverted SL-019 skew)** — `doc/*` is already the *how*;
  the risk is sliding into change-specific design or stale mechanism. The
  exemplar trio + present-tense shipped-obligation rule gate it.
- **`doc/*` duplication tension** — lift durable content, do not retire the
  source; supersession is flagged as a follow-up only (slice non-goal).
  **Interim authority rule (F4):** once a tech spec captures an architecture,
  that spec is authoritative for it and the lifted `doc/*` content is demoted
  to seed/historical (a pointer), even though physical retirement is deferred.
  Without this, two co-authoritative homes recreate the "untrusted prose"
  problem PRD-012 §1 exists to kill.
- **Thin-component anaemia** — capability components degenerating to stubs.
  Bar: a component carries its kind-specific config, statuses, and render
  contracts, not just a pointer at the parent.
- **Parallel drift / reservation contention** — SL-019 mitigations carry
  over: lock bar before fan-out; cap fan-out width.
- **Stale anchors at authoring** — `[[source]]` paths verified against `src/`
  at write time; `spec validate` does not check anchor liveness (drift
  capability's job, unbuilt).

## 10. Open questions

- Final spec roster + boundary calls → PHASE-01 (method locked here; roster
  is scaffolding).
- Fan-out mechanism → `/phase-plan`.
- `doc/*` retirement → out of scope; flag as follow-up at `/close`.

## 11. Inquisition dispositions (external — codex/GPT-5.5)

Adversarial review of this design (commits `f15f2ce` + `9eb1d7b`) against
PRD-012 §4 constraints/invariants, PRD-013, ADR-004, and the SL-019 charge
class. Six charges returned; triaged below with evidence. F1 re-opens locked
D1 and **must be resolved by the User before the design re-locks** — the rest
are integrated.

### F1 — Umbrella scope/altitude is incoherent (UPHELD; fatal-for-lock)

*Charges I + IV + reviewer Q1 collapse to one defect.* The root is labelled
"Doctrine entity system" and its body (§3) lifts only entity-engine content —
storage rule, entity-vs-facet taxonomy, identity/reference model, Raw→Entity→
Registry — yet §2 parents the **whole** architecture under it (install, boot,
dispatch, skills distribution) and §6 retrofits SPEC-001 (priority engine /
`cordage`) and SPEC-002 (reconciliation) beneath it "so the corpus forms one
tree, not a forest." Three problems:

1. **False containment.** PRD-012 §3: containment (`parent`) and peering are
   different relations; conflating them "loses the architecture's shape." The
   priority engine is not *contained by* the entity system — parenting it
   there asserts a decomposition that is untrue.
2. **No doctrine mandates one connected tree.** PRD-012: "a root spec has no
   parent and that is valid." A forest is legal; the single-tree goal is
   invented, then enforced with false edges.
3. **Bad seed.** `doc/entity-model.md:3` declares itself **"Status: direction,
   deferred. No action now"** — the *target* model and a spec-driver critique,
   not shipped architecture, and scoped to entities only. Seeding a
   whole-system root from a deferred-direction, entity-scoped doc is an
   altitude error; the §3 "altitude filter" that salvages it is subjective and
   unreproducible.

**Remediation — User's call (two legitimate fixes):**

- **(A) Proper context root** — author a thin whole-system context spec
  ("Doctrine", the tool) whose children are *all* containers; demote "entity
  system" to one container (the entity engine, seeded from `entity-model.md`).
  Correct C4, preserves the legible single tree, and a context spec is a
  synthesis (REQ-085 admits anchor-free context specs) — not a lift of one
  subsystem doc. *Recommended.*
- **(B) Forest** — drop the single-tree goal; allow multiple roots; author
  `parent` only where containment is substantively true. Lighter; SPEC-001/002
  stay rootless.

Under both, the umbrella's *content* (entity-engine durable material) belongs
on an entity-engine container, not the root.

**RESOLVED — User chose (A).** Applied to the body: §2 root is now a
whole-system context spec ("Doctrine", the tool) with every container parented
to it; §3 exemplar #1 is that synthesis (anchor-free, REQ-085), with
`entity-model.md` content moved down to the entity-engine container (#2) under
the F4 rule; §6 retrofits SPEC-001/002 `parent` = root. D1 re-locked (§1).

### F2 — D2 completeness claim stated two incompatible ways (PARTIALLY UPHELD; minor)

*Charge II.* §1 said "every **active** PRD reachable" while §2/§7 say "every
PRD **whose mechanism ships**". PRD-010 (active, `knowledge_record` unbuilt) is
exempt under the second and a violation under the first. The **exemption is
legitimate** — this slice backfills shipped *how*; forward-intent authoring for
PRD-010 is out of scope, and SPEC-001/002 pre-exist so counting them costs
nothing. Only the wording was inconsistent. **Fixed:** §1 D2 now reads
"shipped-mechanism PRD"; the coverage audit (§7) asserts only that set with
exemptions named. No new spec for PRD-010. *Reviewer's "ad-hoc escape hatch"
framing refuted — the rule is coherent once worded once.*

### F3 — "Locked" vs PHASE-01-deferred boundary (PARTIALLY UPHELD; minor)

*Charge III.* §1 calls D1/D2 locked; §8 still settles "slices: component vs
container," the PRD-010 subset, and CLI-surface inclusion in PHASE-01. No
contradiction once the line is crisp: **the topology *shape* (D1) and the
descent *rule* (D2) are locked; specific node *placement* (the roster) is
PHASE-01's** — which is why the §2 tree is labelled "candidate" and the count
is a range. The reviewer is right that this must not read as committed
topology. No structural change; the candidate-tree caveat in §2 already carries
it, reinforced here.

### F4 — `doc/*` left co-authoritative (PARTIALLY UPHELD; serious)

*Charge V.* Retiring `doc/*` is legitimately out of scope, but §9 ("do not
retire the source") left two durable homes for one architecture — the very
"untrusted prose" PRD-012 §1 exists to kill. *Reviewer's literal "never two
parallel surfaces" cite actually governs hand-vs-import code-anchor
convergence, not doc-vs-spec — but the §1 motivation makes the concern real.*
**Fixed:** §9 now carries an interim authority rule — a tech spec, once
authored, is authoritative for its architecture and the lifted `doc/*` content
is demoted to seed/pointer, physical retirement still deferred.

### F5 — Late `parent` retrofit on active SPEC-001 (REFUTED as framed; folds into F1)

*Charge VI.* Adding a structural `parent` edge is **not** a PRD-013
requirement-reconciliation event — PRD-013 governs requirement *authored-status
vs coverage*, not structural relations. Retrofitting `parent` in PHASE-05 is a
mechanical single-field add. The only live question is whether the edge's
containment is *true* — which is **F1**. Once F1 settles the root scope, the
SPEC-001/002 retrofit is valid. No separate remediation.

### Verified, no change

- **D4 `pending` is correct** (reviewer Q4 resolved). `pending` is the initial
  value of the *authored/normative* requirement-status enum
  (`pending → in-progress → active → deprecated → retired | superseded`,
  spec-entity-spec § Lifecycle; "*pending*: declared, not started"), **not** a
  coverage value. D4 asserts authored status, not derived coverage — coherent
  with PRD-013's two-tier model.
- **Anchors not treated as currency** — §9 already honours the PRD-012
  invariant ("a code anchor is not proof the spec is current"; `validate` does
  not check anchor liveness). No charge.
- **Embed/refresh ritual** (§4: `skills install` + `touch src/skills.rs` +
  rebuild) correct per `mem.pattern.distribution.skill-refresh-command`.
