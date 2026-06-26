# Design SL-159: Epistemic kind catalog: add EVD + HYP

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10). -->

> Source: **RFC-009** (epistemic records as the human-facing relational substrate).
> Carries the two *locked-in-draft* kind **additions** (EVD, HYP). The third
> RFC-009 catalog change — **CON → INV** — was **split out to SL-160** (2026-06-27)
> because its `waived → relaxed` semantics are unsettled and should not block the
> two clean additions. The RFC's broader open deliberation (D2 corpus survey, the
> `shapes` epistemic-vs-affects split, D4 concept-map reify, Tier 2) stays out.
> Governance axis: a **Revision** (ADR-013) is cut **after** this design and settled
> in reconciliation.

## 1. Design Problem

Doctrine's epistemic record taxonomy is four kinds — assumption (ASM), decision
(DEC), question (QUE), constraint (CON). RFC-009 adds two:

- **EVD (evidence)** — a captured datum with provenance that **supports** or
  **disputes** other records. A role, not a topic.
- **HYP (hypothesis)** — a testable proposed answer to a question, distinct from
  QUE (the unsettled matter) and ASM (proceed-as-if-true).

"Fully modelled" (user): the kinds land able to do their job — EVD's
`supports`/`disputes` edges are **in scope**, not deferred. Only the broader D3
surface (the `shapes` role split, concept-map edge types) stays open.

**CON stays unchanged here.** EVD `supports`/`disputes` targets the `RECORD` family,
which includes CON in the interim; when SL-160 renames CON→INV, those edges carry
through unchanged. No dependency in this direction — SL-160 sequences `after` this
slice (shared touch-site files, serial edits).

## 2. Current State

`src/knowledge.rs` (~2.4k lines) is the kind-specific module over the kind-blind
`crate::entity` engine. Four `RecordKind`s each ride an `entity::Kind` const with
its own tree, reservation namespace, prefix, status vocabulary, typed `[facet]`,
and scaffold template. The "add a kind" surface is **not** as centralised as it
looks — ~17 sites hardcode the record-prefix literals rather than reading
`kinds::RECORD` (see `mem.pattern.doctrine.record-kind-touch-sites`). Adding EVD/HYP
touches every one:

- **`src/kinds.rs`** — prefix consts; `RECORD = &[ASM, DEC, QUE, CON]` grouping →
  append `EVD, HYP`.
- **`src/knowledge.rs`** — `RecordKind` enum; per-kind `Kind` const; the
  `kind()`/`as_str()`/`statuses()`/`hidden()`/`terminal()`/scaffold-template/
  `validate_facet()`/`render_facet()`/`format_facet()`/`facet_json()` match arms;
  `RecordKind::ALL: [_; 4]` → `[_; 6]`; per-kind facet struct + `RecordFacet`
  variant; the kind-blind `RawFacet` superset; closed facet value-enums (new
  `Provenance`; `Confidence` reused); `resolve_ref` diagnostic; tests (`ctx_for`,
  `populated_fixture`, vocab/prefix-count/terminal assertions).
- **`src/integrity.rs`** — `KINDS` identity table (+2 rows); `kinds_table_*`
  literal pin (advisory, not enforced); **and** a **third** in-file site: the
  prefix-collision list (`:817`) gains EVD/HYP.
- **`src/priority/partition.rs`** — one `KindPartition` row per record kind (+2;
  `:609` guard list gains EVD/HYP).
- **`src/relation.rs`** — `RELATION_RULES`; `RECORD` const drives `supersedes`/
  `shapes`/`spawns` **source** sets, but the `Shapes` **target** set and
  `GovernedBy` **source** set **hardcode** `ASM, DEC, QUE, CON` → gain `EVD, HYP`.
- **`src/supersede.rs`** — `supersede_policy` + `validate_matrix` record arms (add
  EVD; HYP excluded); **and `src/commands/supersede.rs`** — the command shell.
- **`src/commands/dep_seq.rs`** — `is_record` (`:29`) hardcodes the record prefix
  match + pin test (`:267,:273`), admissible vector (`:285`), and a user-facing
  message (`:83`); all gain EVD/HYP.
- **`src/relation_graph.rs`** — record-keyed edge-emission tests.
- **`src/catalog/scan.rs`** — `outbound_for` dispatch (`:62`) routes the record
  family (`"ASM"|"DEC"|"QUE"|"CON"`) to `knowledge::relation_edges`; the fallthrough
  is `debug_assert!(false, …)` (`:88`). A `KINDS` row with **no scan arm panics
  every debug-build corpus scan** — so adding EVD/HYP rows to `integrity::KINDS`
  *without* extending this arm is a panic, not a silent gap. **Not caught by any
  drift canary** (codex-2 F1).
- **`src/catalog/test_helpers.rs`** — `seed_knowledge` (`:119`) maps record prefix
  → tree dir; needs `EVD|HYP` arms (codex-2 F4, test-only).
- **`src/search.rs`** — hardcodes the knowledge prefix group
  `("knowledge", &["ASM","DEC","QUE","CON"])` + flat lists; EVD/HYP **unsearchable**
  until added (codex F3).
- **`src/tag.rs`** — hardcodes the taggable-prefix list; new kinds **untaggable**
  until added (codex F3).
- **`tests/e2e_knowledge_cli_golden.rs`, `tests/e2e_memory_anchoring.rs`** — e2e
  goldens pinned to the kind catalog (catalog listing + help strings shift with +2
  kinds; codex F6).
- **`install/templates/knowledge-evidence.toml`, `…-hypothesis.toml`** — two new
  seed templates.
- **Docs / shipped memory** — `using-doctrine.md`, glossary,
  `mem.signpost.doctrine.knowledge` (document the two new kinds).

### Built on the landed SL-158 (trinary actionability)

SL-158 (commit `5dd1715c`, merged to edge/main) turned the priority partition
trinary: `priority::partition::KindPartition` carries a `gating` set between
`workable` and `terminal`; the partition-cover canary is
`workable ∪ gating ∪ terminal == <KIND>_STATUSES`. `commands/dep_seq.rs` grew
`is_admissible_dep_target = is_work_like ∨ is_record`, where `is_record` (`:29`)
**hardcodes** `matches!(prefix, "ASM"|"DEC"|"QUE"|"CON")` with a twin pin test.
SL-159 edits these to add EVD/HYP.

Consequence: EVD/HYP gate **correctly on arrival** — a work item can
`needs → EVD-captured` and stay blocked until the EVD is `confirmed`. The kinds are
not inert.

## 3. Forces & Constraints

- **Behaviour-preservation gate** (AGENTS.md): the entity engine is shared
  machinery; existing record suites are the proof and must stay green.
- **No parallel implementation**: ride the existing `RecordKind` checklist, the
  existing `link`/`status`/`new` verbs, the existing supersede transition — add no
  second mechanism. `confidence` reuses the existing `Confidence` enum.
- **Crisp-edge bar** (RFC-009 D1): each new kind names a role/shape with hard edges;
  no kind becomes a parallel implementation of another. EVD names a role (evidence
  *about* records); HYP names a shape (a testable proposed answer) distinct from QUE
  and ASM.
- **Pure/imperative split**: no clock/rng/git/disk in the pure layer (scaffold,
  validate, render stay pure; the date is passed in).
- **ADR-001 layering**: `kinds.rs` is leaf; `relation.rs`/`knowledge.rs` are
  engine/command — no cycle introduced.
- **Shared touch-set with SL-160**: both slices edit the ~17 hardcoded prefix sites.
  SL-159 lands first (these additions); SL-160 (CON→INV) rebases. Serial — no
  parallel edits to the same lines.

## 4. Guiding Principles

The checklist is mechanical but **not** auto-canaried — correctness comes from doing
**every** site (grep, don't trust "centralised") and letting the drift canaries
(vocab/known-set/partition-cover/prefix-count) catch the structured omissions. The
literal match-arm sites (`scan.rs`, `dep_seq.rs`, `search.rs`, `tag.rs`) have no
canary — grep finds them. Prefer the existing seam over a new verb.

## 5. Proposed Design

### 5.1 System Model

`RecordKind` goes from 4 to 6 variants: `Assumption, Decision, Question, Constraint,
Evidence, Hypothesis` (Evidence/Hypothesis append; Constraint unchanged).
`RECORD = &[ASM, DEC, QUE, CON, EVD, HYP]`. `RecordKind::ALL: [_; 6]`. New prefixes
`EVD`, `HYP` in `kinds.rs`.

### 5.2 Interfaces & Contracts

**No new CLI verbs.** Everything rides existing seams:

| intent | verb (existing) |
|---|---|
| author a kind | `knowledge new evidence\|hypothesis …` (new `ValueEnum` variants) |
| transition status | `knowledge status EVD-1 confirmed` (kind-blind; validates per-kind vocab) |
| author `supports`/`disputes` | `link EVD-1 disputes HYP-3` (new `Writable` labels) |
| supersede | `supersede OLD NEW` (existing transition; new EVD arm) |

**New relation labels** (`src/relation.rs`) — full plumbing, not just rows (codex F5):

- `RelationLabel::Supports`, `RelationLabel::Disputes` enum variants, placed at the
  **declaration-order slot** the VT-1 order pin expects (new variants land at their
  source kind's axis-run tail; the test regroups by enum `Ord`).
- `name()` / parse coverage + `inbound_name` reciprocals `supported_by` /
  `disputed_by`.
- `RELATION_RULES` rows: `sources: &[EVD]`, `target: Kinds(RECORD)`, `tier: One`,
  `link: Writable`, `role: None`. EVD is the sole author (RFC: "EVD names a role").
  Target is the **record family only** (incl CON in the interim; widening to RSK
  deferred). EVD→HYP, EVD→CON, EVD→QUE/ASM/DEC/EVD all legal.
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
earlier "rides the existing overlay" claim was wrong — the render list is literal.

**Hardcoded RELATION_RULES lists updated**: the `Shapes` target set and `GovernedBy`
source set gain `EVD, HYP` (CON unchanged).

### 5.3 Data, State & Ownership

**Status vocabularies** (seed = first element):

| kind | STATUSES (seed first) | gating (unsettled) | terminal (settled) | is_terminal (supersession-final) | hidden |
|---|---|---|---|---|---|
| EVD | `captured, disputed, confirmed, retracted, superseded` | `captured, disputed` | `confirmed, retracted, superseded` | `retracted, superseded` | `confirmed, retracted, superseded` |
| HYP | `proposed, confirmed, refuted` | `proposed` | `confirmed, refuted` | `confirmed, refuted` | `confirmed, refuted` |

Notes:
- **EVD `confirmed` is deliberately NOT `is_terminal`** — so the supersede verb can
  flip a `confirmed` EVD to `superseded` (RFC: confirmed may be re-disputed *or*
  superseded). `retracted`/`superseded` are the truly-final states. `superseded` is
  added to EVD's vocab so the existing supersede transition has a landing state.
- Three distinct per-kind subsets persist (pre-existing design): `hidden` (list
  default-hide), `terminal`/`is_terminal` (supersession guard), and the SL-158
  `gating`/`terminal` partition. Independent; each gets a row.

**Facets** (typed `[facet]`, kind-dispatched; every field `"" / [] → absent`):

- `EvidenceFacet { datum: Option<String>, provenance: Option<Provenance>,
  confidence: Option<Confidence> }`. New closed enum `Provenance { Inspection,
  Experiment, Reproduction, Citation }` (kebab serde + `as_str` + `KNOWN`
  drift-canary, mirroring `Basis`). `confidence` **reuses** the existing `Confidence`
  enum. `supports`/`disputes` are **edges, not facet fields**.
- `HypothesisFacet { proposition: Option<String>, predicts: Option<String> }`.
  RFC's candidate `tested_by` is **dropped** — derivable from the inbound
  `supported_by`/`disputed_by` edges (DRY; don't store what the edge yields).

These add fields to the kind-blind `RawFacet` superset (`datum, provenance,
proposition, predicts`) and arms to
`validate_facet`/`render_facet`/`format_facet`/`facet_json`.

**Engine `Kind` consts**: add `EVIDENCE_KIND` (dir `.doctrine/knowledge/evidence`,
prefix `EVD`), `HYPOTHESIS_KIND` (dir `…/hypothesis`, prefix `HYP`).
`integrity::KINDS` +2 rows + pin update.

### 5.4 Lifecycle, Operations & Dynamics

**Supersession** (`src/supersede.rs`): `supersede_policy` — add `EVD`
(`superseded_status: "superseded"`); **HYP excluded** (`None` — RFC silent on HYP
supersession; a refuted HYP is terminal, no use case now). `validate_matrix` extends
to same-kind supersession for EVD. (CON's arm is untouched here; SL-160 renames it.)

**No seed migration** — EVD/HYP are pure additions; no existing data moves.

### 5.5 Invariants, Assumptions & Edge Cases

- **VT canaries gate the structured checklist**: per-kind `statuses` known-set, the
  facet-enum drift canaries (+ a new `Provenance` one), the SL-158 partition-cover
  canary (now over 6 kinds), the prefix-count pin (4 → 6), and the byte-stable
  round-trip per kind. An omitted *structured* site trips one of these.
- **The literal match-arm sites have no canary** — `scan.rs`, `dep_seq.rs`,
  `search.rs`, `tag.rs`, `integrity.rs:817`. Grep is the only guard (R1).
- **`integrity::KINDS` pin is advisory** — must be hand-updated.
- **EVD/HYP both carry `confirmed`** — fine; vocab is per-kind, `union_statuses`
  dedups for the cross-kind `--status` filter.
- **`Shapes` target now includes EVD/HYP** — a record may `shapes` another record
  (existing behaviour for ASM↔record; extends to the new kinds).
- No clock/disk in the pure scaffold/validate/render paths (date passed in).

## 6. Open Questions & Unknowns

- **OQ-1** — `is_record` (`dep_seq.rs:29`) and the partition rows hardcode the
  prefix list rather than reading `kinds::RECORD`. **Out of scope** (IMP-184 tracks
  the DRY refactor); this slice adds EVD/HYP at every site.
- **OQ-2** — Should `Provenance` carry a free-text escape (e.g. an `other` + detail)
  or stay a closed 4-set? Default closed (crisp-edge bar); `datum` holds detail.
  Revisit if it feels narrow in use.

## 7. Decisions, Rationale & Alternatives

- **D1 — fully-modelled, not catalog-only.** EVD's `supports`/`disputes` land now.
  *Alt rejected:* defer edges (EVD inert beyond gating). User chose full modelling;
  the edges ride `link` cheaply.
- **D2 — `supports`/`disputes` are `Writable` `link` edges, manual transitions.**
  *Alt rejected:* `LifecycleOnly` + an evidence→status automation verb (RFC floated
  it). Heavier; conflicts with author's-judgment posture; deferred.
- **D4 — target breadth = RECORD-only** (not RECORD ∪ RSK). Crisp; widen later if
  the risk-substantiation need is real.
- **D5 — drop HYP `tested_by` facet** in favour of the inbound edge (DRY).
- **D7 — HYP not supersedable** (supersede_policy `None`); EVD supersedable.
- **(split)** — CON→INV moved to **SL-160** (`after` this slice). Its `waived →
  relaxed` semantic question was unsettled and would have blocked these additions.

## 8. Risks & Mitigations

- **R1 — a hardcoded literal site is missed.** The literal match-arm sites
  (`scan.rs:62`, `dep_seq.rs`, `search.rs:33`, `tag.rs:17`, `integrity.rs:817`) have
  **no drift canary**; `scan.rs` omission is a debug-build panic. *Mitigation:* grep
  every record-prefix cluster (`mem.pattern.doctrine.record-kind-touch-sites`) and
  confirm EVD/HYP present before close; the partition-cover + prefix-pin + KINDS-pin
  canaries catch the structured sites.
- **R2 — SL-160 (CON→INV) edits the same lines.** *Mitigation:* SL-159 lands first;
  SL-160 `after` it, rebases on the EVD/HYP-extended sites. Serial — report-and-halt
  on any conflict, never parallel.
- **R3 — `mem.signpost.doctrine.knowledge` (shipped) drifts** — documents 4 kinds.
  *Mitigation:* update + re-embed (`cargo build`) + `memory sync` in the docs step.

## 9. Quality Engineering & Validation

Red/green/refactor, behaviour-preservation gate, `just gate` zero-warnings. New /
revised VTs:

- per-kind status known-set for EVD/HYP (extends the existing table test).
- `Provenance` known-set drift canary (new).
- SL-158 partition-cover canary now green over 6 kinds; EVD/HYP `gating` non-empty.
- `supports`/`disputes`: legal author = EVD only; illegal author refused; target ∈
  RECORD; `link`/`unlink` round-trip; `show` renders the edge + reciprocal.
- byte-stable round-trip per new kind (the `populated_fixture` arm extends).
- supersede: EVD `confirmed → superseded` flips; **HYP refuses supersession cleanly**
  (the `supersede_policy → None` path — currently untested, all existing kinds
  return `Some`; assert a clean error, not a panic).
- **headline gating (end-to-end):** a work item `needs → EVD-captured` is blocked;
  the EVD `→ confirmed` makes the dependent actionable. Proves the new kinds
  participate in SL-158's trinary gating, not just that the partition rows parse.
- **`supports`/`disputes` render (codex F4):** after `link EVD-1 disputes HYP-2`,
  `knowledge show HYP-2` surfaces the `disputed_by` reciprocal and `knowledge show
  EVD-1` the `disputes` edge — in both table and JSON.
- **search/tag reach the new kinds (codex F3):** `search` finds an EVD/HYP body;
  `tag` sets/clears a tag on each — the hardcoded prefix groups now include them.
- **e2e goldens (codex F6):** `e2e_knowledge_cli_golden.rs` + `e2e_memory_anchoring.rs`
  updated for the 6-kind catalog (+EVD/HYP listing + help strings).

**Tests that flip by design (consumer revision, not regression):** the prefix-count
pin (4→6), the `RecordKind::ALL` arity, `relation.rs` hardcoded vectors + the
RelationLabel order/coverage pins (two new labels), the `relation_graph.rs`
edge-emission test, `integrity` `kinds_table_*` pin, the `search.rs`/`tag.rs`
prefix-group tests, and the two e2e goldens.

### Implementation shape (phasing is /plan's job)

Roughly: (1) add EVD + HYP kinds (catalog + facets + partition + integrity +
scan arm + templates); (2) `supports`/`disputes` edges + show wiring; (3) docs +
shipped memory; the **Revision** is cut post-design and settled in reconciliation.

## 10. Review Notes

> **Scope note (2026-06-27):** CON→INV was split out to **SL-160** after the codex
> passes below. Findings about CON-keyed sites (e.g. the CON-001 citations, the
> `waived`/`constraint` literals) moved with it; what remains here is the EVD/HYP
> half. The codex-2 F1 `scan.rs` panic finding applies to **both** halves — this
> slice adds the `EVD|HYP` arm; SL-160 changes the `CON` literal.

### Internal adversarial pass (2026-06-27)

Three substantive findings on the additions, **all resolved**:

- **F1 — `supports` edge-label collides with the `[evidence].supports` facet
  field.** *Resolved: keep RFC's `supports`/`disputes`.* Cross-namespace clash
  (relation label vs free-text field), not a real ambiguity; doctrine reuses
  spellings across tiers. Aside: the typed EVD edges may make the free-text
  `[evidence]` block redundant — a later deprecation question, out of scope.
- **F2 — EVD's 5th status `superseded`.** *Resolved: keep 5 states, EVD
  supersedable.* RFC implies it; evidence-superseded-by-better-evidence is real.
- **F3 — `supports`/`disputes` adjacent to the open D3 `shapes`-split.** *Resolved:
  proceed.* D3's open question is the `shapes` *role* split, not whether
  `supports`/`disputes` exist; EVD-authored evidentiary edges are distinct.

### External adversarial passes (codex, 2026-06-27)

Codex (GPT-5.5) hostile review, two passes; each finding verified against ground
truth before integrating (external reviewers hallucinate paths/lines). Findings
relevant to the EVD/HYP additions:

- **F3 (MAJOR)** — `search.rs:33` + `tag.rs:17` hardcode the 4-kind knowledge prefix
  set; EVD/HYP would be unsearchable/untaggable. → §2 + §9 + selectors. *Verified.*
- **F4 (MAJOR)** — `format_metadata`/`show_json` (`knowledge.rs:1004`,`:1149`)
  hardcode `[Shapes, Spawns, GovernedBy]`; `supports`/`disputes` authorable-but-
  invisible. → §5.2 renderer edit + §9 render VT. *Verified.*
- **F5 (MAJOR)** — new `RelationLabel` is full plumbing (enum, `name()`, parser,
  order pin, canaries), not two rows. → §5.2.
- **F6 (MAJOR)** — e2e goldens shift with the catalog. → §9. *Verified.*
- **F7 (MINOR)** — `src/commands/supersede.rs` distinct from `src/supersede.rs`. → §2.
- **Codex-2 F1 (MAJOR, panic-grade)** — `src/catalog/scan.rs` `outbound_for`
  dispatch (`:62`) + `debug_assert!(false)` fallthrough (`:88`): a `KINDS` row with
  no scan arm panics every debug-build corpus scan. The whole `src/catalog/` module
  was missed by the original §2. → §2 + selectors (`scan.rs`, `test_helpers.rs`).
  *Verified.*
- **Codex-2 F4 (MINOR)** — `catalog/test_helpers.rs:119` `seed_knowledge`. → §2.

Harvested to durable memory: `mem.pattern.doctrine.record-kind-touch-sites` (the
~17-site scatter) + IMP-184 (DRY refactor).

Net: no decision overturned; CON→INV split to SL-160. Design holds; ready to lock.
