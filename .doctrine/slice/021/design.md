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

- **D1 Taxonomy topology** — umbrella context spec + mechanism containers +
  thin capability components (§2).
- **D2 Descent rule: capability-complete** — every active PRD reachable via
  ≥1 `descends_from`; shared substrate carries thin per-capability component
  specs rather than restating mechanism (§2).
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
context   Doctrine entity system            ← doc/entity-model.md seed; no parent, no descent
├─ container  Entity engine                 descent: — (shared substrate)
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
└─ container  Dispatch & worktree           descent: — (ADR-006 governs; no PRD)
   (CLI surface — SL-025 uniform contract: candidate container, descent —)
```

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
  intent satisfies reachability). Retrofit `parent` onto both under the
  umbrella in PHASE-05 (single-field edits) so the corpus forms one tree, not
  a forest.
- **Excluded:** drift ledger (unbuilt, no PRD — backfill covers shipped
  mechanism only).
- ~12–14 new specs (~14–16 corpus incl. SPEC-001/002).

## 3. Exemplar trio (D3)

Three specs authored end-to-end, one per shape, before any fan-out:

1. **Umbrella context spec** — "Doctrine entity system". `c4_level =
   "context"`, no parent, no descent. Lifts `doc/entity-model.md`'s durable
   content: the storage rule, entity-vs-facet taxonomy, identity/reference
   model, family-specific status vocab, runtime-state boundary, three-layer
   Rust model (Raw → Entity → Registry). **Altitude filter:** lift shipped
   reality only — entity-model.md's direction/migration/adjudication content
   (roadmap, importer stance, spec-driver critique) is *not* shipped how and
   stays in `doc/*`. Anchors none/minimal (REQ-085 admits anchor-free context
   specs). Requirements: NF-flavoured invariants (storage rule, outbound-only
   relations).
2. **Entity-engine container** — parent = umbrella, descent empty,
   `c4_level = "container"`. Sources: `src/entity.rs` + registry/integrity
   seams (`integrity::KINDS`), `doc/relation-index.md`,
   mem.system.engine.identity-claim-seam,
   mem.pattern.entity.kind-is-data-not-trait. The commonest fan-out shape.
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

- `descends_from` per the §2 tree; umbrella + shared-substrate containers
  stay empty.
- `parent` wiring per tree; **retrofit SPEC-001/SPEC-002** under the umbrella
  so the corpus is one tree.
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
2. **PHASE-02 Exemplar trio** — umbrella + engine container + ADR component;
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
