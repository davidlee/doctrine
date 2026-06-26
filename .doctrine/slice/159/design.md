# Design SL-159: Epistemic kind catalog: add EVD + HYP, replace CON with INV

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

> Source: **RFC-009** (epistemic records as the human-facing relational substrate).
> Carries the three *locked-in-draft* kind-catalog changes; the RFC's open
> deliberation (D2 corpus survey, the `shapes` epistemic-vs-affects split, D4
> concept-map reify, Tier 2) stays out. Decisions locked with the user in the
> `/design` pass (2026-06-27). Governance axis: a **Revision** (ADR-013) is cut
> **after** this design and settled in reconciliation — not authored now.

## 1. Design Problem

Doctrine's epistemic record taxonomy is four kinds — assumption (ASM), decision
(DEC), question (QUE), constraint (CON). RFC-009 lands two additions and one
replacement, all decided:

- **EVD (evidence)** — a captured datum with provenance that **supports** or
  **disputes** other records. A role, not a topic.
- **HYP (hypothesis)** — a testable proposed answer to a question, distinct from
  QUE (the unsettled matter) and ASM (proceed-as-if-true).
- **CON → INV (invariant)** — replace "a boundary that must not be crossed" with
  "a property that must hold." Near-duals; the crisp-edge bar admits one framing,
  and INV is the engineering-appropriate one that composes with EVD (an EVD can
  `disputes` an INV — evidence the property was violated).

"Fully modelled" (user, this pass): the kinds land able to do their job — EVD's
`supports`/`disputes` edges are **in scope**, not deferred. Only the broader D3
surface (the `shapes` role split, concept-map edge types) stays open.

## 2. Current State

`src/knowledge.rs` (~2.4k lines) is the kind-specific module over the kind-blind
`crate::entity` engine. Four `RecordKind`s each ride an `entity::Kind` const with
its own tree, reservation namespace, prefix, status vocabulary, typed `[facet]`,
and scaffold template. The "add a kind" surface is a checklist threaded through
~13 sites, all centralised:

- **`src/kinds.rs`** — prefix consts; `RECORD = &[ASM, DEC, QUE, CON]` grouping.
- **`src/knowledge.rs`** — `RecordKind` enum; per-kind `Kind` const; the
  `kind()`/`as_str()`/`statuses()`/`hidden()`/`terminal()`/scaffold-template/
  `validate_facet()`/`render_facet()`/`format_facet()`/`facet_json()` match arms;
  `RecordKind::ALL: [_; 4]`; per-kind facet struct + `RecordFacet` variant;
  the kind-blind `RawFacet` superset; closed facet value-enums (`Confidence`,
  `Basis`, `ConstraintSource`); `resolve_ref` diagnostic; tests (`ctx_for`,
  `populated_fixture`, vocab/prefix-count/terminal assertions).
- **`src/integrity.rs`** — `KINDS` identity table (records present since SL-059);
  `kinds_table_*` literal pin (advisory, not enforced —
  `mem.pattern.entity.numbered-kind-identity-table`).
- **`src/priority/partition.rs`** — one `KindPartition` row per record kind.
- **`src/relation.rs`** — `RELATION_RULES`; `RECORD` const drives `supersedes`/
  `shapes`/`spawns` **source** sets, but the `Shapes` **target** set and
  `GovernedBy` **source** set **hardcode** `ASM, DEC, QUE, CON`.
- **`src/supersede.rs`** — `supersede_policy` + `validate_matrix` record arms;
  **and `src/commands/supersede.rs`** — the command shell + CON-001 fixtures.
- **`src/commands/dep_seq.rs`** — `is_record` (`:29`) hardcodes the record prefix
  match + its pin test (`:264-273`); add EVD/HYP, CON→INV (SL-158 dependency).
- **`src/relation_graph.rs`** — CON-keyed edge-emission tests.
- **`src/search.rs`** — hardcodes the knowledge prefix group
  `("knowledge", &["ASM","DEC","QUE","CON"])` + two flat prefix lists; EVD/HYP/INV
  are **unsearchable** until added (codex F3).
- **`src/tag.rs`** — hardcodes the taggable-prefix list incl. `CON`; new kinds
  **untaggable** until added (codex F3).
- **`tests/e2e_knowledge_cli_golden.rs`, `tests/e2e_memory_anchoring.rs`** — e2e
  goldens pinned to `constraint`/`CON` (codex F6).
- **`install/templates/knowledge-*.toml`** — one seed template per kind.
- **Docs / shipped memory** — `using-doctrine.md`, glossary, `seed-onboarding.md`,
  `mem.signpost.doctrine.knowledge`.
- **Seed corpus** — `CON-001` (the shipped constraint seed).

### Dependency on SL-158 (lands first)

SL-158 (Trinary actionability, phases nearly complete) changes the shared surface
this slice extends; **SL-159 rebases on the landed SL-158**:

- `priority::partition::KindPartition` gains a third set, `gating`, between
  `workable` and `terminal` (unsettled record → non-`Workable`, non-`Terminal`
  `Gating` class). The records move their unsettled states into `gating`. The VT
  canary generalises to `workable ∪ gating ∪ terminal == <KIND>_STATUSES`.
- `commands/dep_seq.rs` grew `is_admissible_dep_target = is_work_like ∨ is_record`
  — a work item may `needs`/`after` a record. **OQ-1 resolved against merged code:
  `is_record` (dep_seq.rs:29) HARDCODES `matches!(prefix, "ASM"|"DEC"|"QUE"|"CON")`**
  with a twin pin test (`:264-273`). SL-159 must edit both (add EVD/HYP, CON→INV) —
  `dep_seq.rs` is a touch-site.
- `RECORD` gains `references` (concerns-role) authoring.

**SL-158 has landed** (commit `5dd1715c`, merged to edge/main). The merged
`KindPartition` carries the `gating` field and the record rows match this design's
§5.3 table exactly (ASM `gating:[held,testing]`; CON `gating:[active],
terminal:[waived,superseded,retired]` → INV becomes `gating:[active],
terminal:[relaxed,superseded,retired]`). Partition design verified against real
code, not just SL-158's design doc.

Consequence: EVD/HYP gate **correctly on arrival** — a work item can
`needs → EVD-captured` and stay blocked until the EVD is `confirmed`. The kinds
are not inert.

## 3. Forces & Constraints

- **Behaviour-preservation gate** (AGENTS.md): the entity engine is shared
  machinery; existing record suites are the proof and must stay green (adjusted
  for the rename, never broken).
- **No parallel implementation**: ride the existing `RecordKind` checklist, the
  existing `link`/`status`/`new` verbs, the existing supersede transition — add
  no second mechanism. `confidence` reuses the existing `Confidence` enum.
- **Crisp-edge bar** (RFC-009 D1): each new kind names a role/shape with hard
  edges; no kind becomes a parallel implementation of another.
- **CON→INV is a destructive rename of a shipped kind** — tree dir, reservation
  namespace, seed record, templates, and every literal `"CON"`/`Constraint` site
  move together or integrity breaks.
- **Pure/imperative split**: no clock/rng/git/disk in the pure layer (scaffold,
  validate, render stay pure; the date is passed in).
- **ADR-001 layering**: `kinds.rs` is leaf; `relation.rs`/`knowledge.rs` are
  engine/command — no cycle introduced.

## 4. Guiding Principles

The checklist is mechanical and centralised; correctness comes from doing **every**
site and letting the drift canaries (vocab/known-set/partition-cover/prefix-count)
catch omissions. Prefer the existing seam over a new verb. Keep CON→INV a faithful
rename plus the single agreed semantic nudge (`waived`→`relaxed`).

## 5. Proposed Design

### 5.1 System Model

`RecordKind` goes from 4 to 6 variants: `Assumption, Decision, Question, Invariant,
Evidence, Hypothesis` (Invariant takes Constraint's slot; Evidence/Hypothesis
append). `RECORD = &[ASM, DEC, QUE, INV, EVD, HYP]`. `RecordKind::ALL: [_; 6]`.
New prefixes `EVD`, `HYP`, `INV` in `kinds.rs` (CON retired; **its prefix const is
removed**, not recycled — RFC-009 D4: recycling CON would mislead).

### 5.2 Interfaces & Contracts

**No new CLI verbs.** Everything rides existing seams:

| intent | verb (existing) |
|---|---|
| author a kind | `knowledge new evidence\|hypothesis\|invariant …` (new `ValueEnum` variants) |
| transition status | `knowledge status EVD-1 confirmed` (kind-blind; validates per-kind vocab) |
| author `supports`/`disputes` | `link EVD-1 disputes HYP-3` (new `Writable` labels) |
| supersede | `supersede OLD NEW` (existing transition; new arms) |

**New relation labels** (`src/relation.rs`) — full plumbing, not just rows (codex F5):

- `RelationLabel::Supports`, `RelationLabel::Disputes` enum variants, placed at the
  **declaration-order slot** the VT-1 order pin expects (new variants land at their
  source kind's axis-run tail; the test regroups by enum `Ord`).
- `name()` / parse coverage + `inbound_name` reciprocals `supported_by` /
  `disputed_by`.
- `RELATION_RULES` rows: `sources: &[EVD]`, `target: Kinds(RECORD)`, `tier: One`,
  `link: Writable`, `role: None`. EVD is the sole author (RFC: "EVD names a role").
  Target is the **record family only** (epistemic targets; widening to RSK
  deferred). EVD→HYP, EVD→INV, EVD→QUE/ASM/EVD all legal.
- The source/target/tier/coverage canaries that pin every label extend to the two
  new rows.
- Transitions stay **manual** via `status` — `supports`/`disputes` do **not**
  auto-flip the target (no evidence→status automation engine; author's judgment,
  per RFC's EVD-reopen note).

**Knowledge display renderers must emit the new edges (codex F4).**
`format_metadata` (`knowledge.rs:1004`) and `show_json` (`:1149`) **hardcode**
`[Shapes, Spawns, GovernedBy]` — `supports`/`disputes` would be authorable but
**invisible**. Add both labels to those two renderers (a record that is
`supported_by`/`disputed_by` should also surface the inbound reciprocal). The
earlier design's "rides the existing overlay" claim was wrong — the render list is
literal, not generic.

**Hardcoded RELATION_RULES lists updated** (the RFC's "no table change" was wrong):
`Shapes` target set and `GovernedBy` source set drop `CON`, gain `INV, EVD, HYP`.

### 5.3 Data, State & Ownership

**Status vocabularies** (seed = first element):

| kind | STATUSES (seed first) | gating (unsettled) | terminal (settled) | is_terminal (supersession-final) | hidden |
|---|---|---|---|---|---|
| EVD | `captured, disputed, confirmed, retracted, superseded` | `captured, disputed` | `confirmed, retracted, superseded` | `retracted, superseded` | `confirmed, retracted, superseded` |
| HYP | `proposed, confirmed, refuted` | `proposed` | `confirmed, refuted` | `confirmed, refuted` | `confirmed, refuted` |
| INV | `active, relaxed, superseded, retired` | `active` | `relaxed, superseded, retired` | `relaxed, superseded, retired` | `relaxed, superseded, retired` |

Notes:
- **EVD `confirmed` is deliberately NOT `is_terminal`** — so the supersede verb can
  flip a `confirmed` EVD to `superseded` (RFC: confirmed may be re-disputed *or*
  superseded). `retracted`/`superseded` are the truly-final states. `superseded`
  is added to EVD's vocab so the existing supersede transition has a landing state.
- **INV** = CON's vocab with `waived → relaxed`. The facet's waiver fields rename
  (below). `superseded`/`retired`/`active` unchanged.
- Three distinct per-kind subsets persist (pre-existing design): `hidden`
  (list default-hide), `terminal`/`is_terminal` (supersession guard), and the
  SL-158 `gating`/`terminal` partition. They are independent and each get a row.

**Facets** (typed `[facet]`, kind-dispatched; every field `"" / [] → absent`):

- `EvidenceFacet { datum: Option<String>, provenance: Option<Provenance>,
  confidence: Option<Confidence> }`. New closed enum `Provenance {
  Inspection, Experiment, Reproduction, Citation }` (kebab serde + `as_str` +
  `KNOWN` drift-canary, mirroring `Basis`). `confidence` **reuses** the existing
  `Confidence` enum. `supports`/`disputes` are **edges, not facet fields**.
- `HypothesisFacet { proposition: Option<String>, predicts: Option<String> }`.
  RFC's candidate `tested_by` is **dropped** — derivable from the inbound
  `supported_by`/`disputed_by` edges (DRY; don't store what the edge yields).
- `InvariantFacet` = renamed `ConstraintFacet`: `statement, source(InvariantSource),
  applies_to[], relaxation_reason, relaxed_by, relaxed_on` (was `waiver_reason,
  waived_by, waived_on`). `ConstraintSource → InvariantSource`, variants unchanged
  (`canon, adr, external, technical, legal, compatibility, operator`).

These add fields to the kind-blind `RawFacet` superset (`datum, provenance,
proposition, predicts`, and the `waiver_* → relaxation_*` renames) and arms to
`validate_facet`/`render_facet`/`format_facet`/`facet_json`.

**Engine `Kind` consts**: rename `CONSTRAINT_KIND → INVARIANT_KIND`
(dir `.doctrine/knowledge/invariant`, prefix `INV`); add `EVIDENCE_KIND`
(dir `…/evidence`, prefix `EVD`), `HYPOTHESIS_KIND` (dir `…/hypothesis`, prefix
`HYP`). `integrity::KINDS` rename + two new rows + pin update.

### 5.4 Lifecycle, Operations & Dynamics

**Supersession** (`src/supersede.rs`): `supersede_policy` — rename the `CON` arm to
`INV` (`superseded_status: "superseded"`); add `EVD` (`superseded_status:
"superseded"`); **HYP excluded** (`None` — RFC silent on HYP supersession; a refuted
HYP is terminal, no use case now). `validate_matrix` extends to same-kind
supersession for INV and EVD.

**CON→INV seed migration** (DD-F, decided: **in-place rewrite**, not supersede —
the CON kind ceases to exist, so there is nothing to supersede *into*):
- move `.doctrine/knowledge/constraint/001/` → `.doctrine/knowledge/invariant/001/`
- rewrite `record-001.toml`: `record_kind = "invariant"`; facet field renames if
  populated; status (`active`) unchanged.
- rename the `001-<slug>` symlink; rewrite the `.md` canonical ref `CON-001 → INV-001`.
- reservation namespace: `INV` mints fresh from id 2 above the seed (the seed is id 1).
- the CON tree dir is removed.

### 5.5 Invariants, Assumptions & Edge Cases

- **VT canaries gate the checklist**: per-kind `statuses` known-set, the three
  facet-enum drift canaries (+ a new `Provenance` one), the SL-158 partition-cover
  canary (now over 6 kinds), the prefix-count pin (4 → 6), and the byte-stable
  round-trip per kind. An omitted site trips one of these.
- **`integrity::KINDS` pin is advisory** — must be hand-updated; nothing else
  catches a missing row.
- **EVD/HYP both carry `confirmed`** — fine; vocab is per-kind, `union_statuses`
  dedups for the cross-kind `--status` filter.
- **`Shapes` target now includes EVD/HYP/INV** — a record may `shapes` another
  record (existing behaviour for ASM↔record; extends to the new kinds).
- No clock/disk in the pure scaffold/validate/render paths (date passed in).

## 6. Open Questions & Unknowns

- **OQ-1** — ~~Does SL-158's `is_record` read `kinds::RECORD` or hardcode?~~
  **RESOLVED** (SL-158 merged): hardcoded (`dep_seq.rs:29` + pin test `:264-273`).
  Both edited by this slice. *(A latent cleanup — `is_record` and the partition
  rows could read `kinds::RECORD` — is noted as a follow-up, not in scope.)*
- **OQ-2** — Should `Provenance` carry a free-text escape (e.g. an `other` +
  detail) or stay a closed 4-set? Default closed (crisp-edge bar); `datum` holds
  detail. Revisit if it feels narrow in use.
- **OQ-3** — `applies_to` on INV: keep the list as-is (it reads fine for "the
  invariant applies to these paths/modules"). No change proposed.

## 7. Decisions, Rationale & Alternatives

- **D1 — fully-modelled, not catalog-only.** EVD's `supports`/`disputes` land now.
  *Alt rejected:* defer edges (EVD inert beyond gating). User chose full modelling;
  the edges ride `link` cheaply.
- **D2 — `supports`/`disputes` are `Writable` `link` edges, manual transitions.**
  *Alt rejected:* `LifecycleOnly` + an evidence→status automation verb (RFC floated
  it). Heavier; conflicts with author's-judgment posture; deferred.
- **D3 — CON→INV faithful rename + `waived → relaxed`** (and facet `waiver_* →
  relaxation_*`). *Alt rejected:* full invariant-native lifecycle/facet redesign —
  bleeds into the open D3 EVD-disputes-INV loop; over-scoped. *Alt rejected:* pure
  rename keeping `waived` — reads wrong for "a property that must hold."
- **D4 — target breadth = RECORD-only** (not RECORD ∪ RSK). Crisp; widen later if
  the risk-substantiation need is real.
- **D5 — drop HYP `tested_by` facet** in favour of the inbound edge (DRY).
- **D6 — in-place seed rewrite** for CON-001 (the kind is renamed, the record
  migrates with it). *Alt rejected:* supersede (no surviving CON kind to point at).
- **D7 — HYP not supersedable** (supersede_policy `None`); EVD/INV supersedable.

## 8. Risks & Mitigations

- **R1 — destructive CON rename misses a literal site.** *Mitigation:* grep
  `Constraint|CONSTRAINT|"CON"|kinds::CON|/constraint|waived` to zero before close
  (the `waived` literal too — it lives in HIDDEN/TERMINAL arrays + tests); the
  partition-cover + prefix-pin + KINDS-pin canaries catch the structured sites.
- **R5 — orphaned CON reservation ref (reasoning corrected, codex F1).**
  `reserve::remote_reservation_ids` (`reserve.rs:203`) is **NOT prefix-scoped** — it
  unions the trailing `NNN` of *every* `refs/doctrine/reservation/*/NNN` into the
  candidate set for *any* mint. So a stale `CON/001` ref **is** read into INV/EVD/HYP
  allocation (the earlier "INV mints from the tree, CON ref irrelevant" claim was
  wrong). Net effect stays **harmless**: id 1 is ≤ existing maxima and the CAS
  create guards collisions. *Mitigation:* delete `refs/doctrine/reservation/CON/*`
  in the migration for hygiene; not correctness-critical.
- **R6 — authored prose citing `CON-001` would dangle (codex F2, low).** Integrity
  scans authored `.doctrine/**/*.md` for inbound citations and reports danglers with
  a non-zero exit (`integrity.rs:545-566`; disposable `state/` prose is skipped).
  **Grep is clean today** — the only `CON-001` hits are in gitignored
  `.doctrine/state/dispatch/**`. *Mitigation:* re-grep authored tier before close;
  rewrite any `CON-001` → `INV-001` if one appears.
- **R2 — SL-158 not yet landed when execution starts.** *Mitigation:* sequence
  after SL-158 (`git fetch . edge:main` before execute); design targets the landed
  trinary `KindPartition` shape. If SL-158 slips, the partition rows are the only
  blocked sites — the rest proceeds.
- **R3 — seed CON-001 inbound relations orphaned by the rewrite.** *Mitigation:*
  scan for inbound edges to `CON-001` before migration; rewrite referrers to
  `INV-001` in the same step (or confirm none exist — likely, it's a fresh seed).
- **R4 — `mem.signpost.doctrine.knowledge` (shipped) drifts** — it documents 4
  kinds with *stale* status vocabularies. *Mitigation:* update + re-embed +
  `memory sync` in the docs step.

## 9. Quality Engineering & Validation

Red/green/refactor, behaviour-preservation gate, `just gate` zero-warnings. New /
revised VTs:

- per-kind status known-set for EVD/HYP/INV (extends the existing table test).
- `Provenance` known-set drift canary (new); `InvariantSource` canary (renamed).
- SL-158 partition-cover canary now green over 6 kinds; EVD/HYP `gating` non-empty.
- `supports`/`disputes`: legal author = EVD only; illegal author refused; target
  ∈ RECORD; `link`/`unlink` round-trip; `show` renders the edge + reciprocal.
- byte-stable round-trip per new kind (the `populated_fixture` arm extends).
- seed migration: post-rewrite `knowledge show INV-001` succeeds; `CON-001` gone;
  no `constraint` tree remains.
- supersede: EVD `confirmed → superseded` flips; **HYP refuses supersession
  cleanly** (the `supersede_policy → None` path — currently untested, all existing
  kinds return `Some`; assert a clean error, not a panic).
- **headline gating (end-to-end):** a work item `needs → EVD-captured` is blocked;
  the EVD `→ confirmed` makes the dependent actionable. Proves the new kinds
  participate in SL-158's trinary gating, not just that the partition rows parse.
- **`supports`/`disputes` render (codex F4):** after `link EVD-1 disputes HYP-2`,
  `knowledge show HYP-2` surfaces the `disputed_by` reciprocal and `knowledge show
  EVD-1` the `disputes` edge — in both table and JSON.
- **search/tag reach the new kinds (codex F3):** `search` finds an EVD/HYP/INV body;
  `tag` sets/clears a tag on each — the hardcoded prefix groups now include them.
- **e2e goldens (codex F6):** `e2e_knowledge_cli_golden.rs` +
  `e2e_memory_anchoring.rs` updated for the 6-kind catalog + `constraint→invariant`
  rename (these flip by design — listed below).

**Tests that flip by design (consumer revision, not regression):** the prefix-count
pin (4→6), the `statuses(CON)` / `is_terminal(CON)` / partition-CON assertions
(rename to INV + `relaxed`), `relation.rs` hardcoded vectors (1425/1442) + the
RelationLabel order/coverage pins (two new labels), the `relation_graph.rs` CON
edge-emission test, `integrity` `kinds_table_*` pin, `supersede` CON arm test
(src + `src/commands/supersede.rs` fixtures), the `search.rs`/`tag.rs` prefix-group
tests, and the two e2e goldens.

### Implementation shape (phasing is /plan's job)

Roughly: (1) CON→INV rename + seed migration (self-contained, behaviour-preserving);
(2) add EVD + HYP kinds (catalog + facets + partition + integrity + templates);
(3) `supports`/`disputes` edges + show wiring; (4) docs + shipped memory; the
**Revision** is cut post-design and settled in reconciliation.

## 10. Review Notes

### Internal adversarial pass (2026-06-27)

Cheap fixes integrated: narrowed the `memory/**` selector to the one shipped file
(F4); added the `waived` literal to R1's grep (F5); added the headline end-to-end
gating VT + the `supersede → None` clean-refusal VT to §9 (F6); added R5 (orphaned
CON reservation ref).

Three substantive findings surfaced to the user; **all resolved 2026-06-27**:

- **F1 — `supports` edge-label collides with the `[evidence].supports` facet
  field.** *Resolved: keep RFC's `supports`/`disputes`.* Cross-namespace clash
  (relation label vs free-text field), not a real ambiguity; doctrine reuses
  spellings across tiers and users expect the RFC vocabulary. Aside: the typed EVD
  edges may make the free-text `[evidence]` block redundant — a later deprecation
  question, out of scope here.
- **F2 — EVD's 5th status `superseded`.** *Resolved: keep 5 states, EVD
  supersedable.* RFC implies it ("confirmed may be superseded by new contrary
  evidence"); evidence-superseded-by-better-evidence is a real path.
- **F3 — `supports`/`disputes` adjacent to the open D3 `shapes`-split.**
  *Resolved: proceed.* D3's open question is the `shapes` *role* split
  (epistemic-vs-affects), not whether `supports`/`disputes` exist; EVD-authored
  evidentiary edges are distinct and do not preempt that resolution.

### External adversarial pass (codex, 2026-06-27)

Codex (GPT-5.5) hostile review of the design doc. Seven findings; each verified
against ground truth before integrating (external reviewers hallucinate
paths/lines — and two here did over-claim).

**Accepted + integrated:**
- **F3 (MAJOR)** — `src/search.rs` + `src/tag.rs` hardcode the 4-kind knowledge
  prefix set; EVD/HYP/INV would be unsearchable/untaggable. → added to §2 + §9 +
  selectors. *Verified real* (`search.rs:33`, `tag.rs:17`).
- **F4 (MAJOR)** — `format_metadata`/`show_json` hardcode `[Shapes, Spawns,
  GovernedBy]`; `supports`/`disputes` authorable-but-invisible. → §5.2 renderer
  edit + §9 render VT. *Verified real* (`knowledge.rs:1004`, `:1149`); the prior
  "rides the overlay" claim was wrong.
- **F5 (MAJOR)** — new `RelationLabel` is full plumbing (enum variant, `name()`,
  parser, order pin, canaries), not two rows. → §5.2 expanded.
- **F6 (MAJOR)** — e2e goldens + CLI/help/doc strings pinned to `constraint`/`CON`.
  → §9 goldens added. *Verified real* (`tests/e2e_knowledge_cli_golden.rs`,
  `e2e_memory_anchoring.rs`, `using-doctrine.md:50` — which also mis-states the CON
  lifecycle today, a free fix).
- **F7 (MINOR)** — `src/commands/supersede.rs` (distinct from `src/supersede.rs`)
  carries CON-001 fixtures. → §2 + selectors. *Verified real.*

**Accepted with corrected severity:**
- **F1 (claimed CRITICAL → MAJOR/reasoning):** reservation ids are pooled, not
  prefix-scoped (`reserve.rs:203`), so R5's rationale was wrong — but the effect is
  still harmless (CAS-guarded, id 1 ≤ maxima). → R5 reasoning corrected; ref
  cleanup added as hygiene.

**Over-claimed (dismissed with evidence):**
- **F2 (claimed MAJOR):** the named authored files (`question/001/record-001.md:26`,
  `adr-017.md:21`) do **not** cite `CON-001` (grep-clean); the only hits are
  gitignored `state/dispatch/**`, which integrity skips. The real guard (authored
  prose dangling) is genuine-but-vacuous today → captured as R6 (re-grep before
  close), not a touch-site.

Net: the touch surface grew from ~13 to ~18 sites; no decision (D1–D7) overturned.
Design holds; ready to lock.

