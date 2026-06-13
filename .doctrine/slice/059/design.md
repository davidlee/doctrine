# Design SL-059: Knowledge records — the standalone four-kind entity surface

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-059, SPEC-019, ADR-004, REQ-239); doc-local refs bare — D1 (§4),
     L1 (§4), OQ1 (§9), R1 (§13). -->

## 1. Design Problem

SPEC-019 (amended `6643f4c`, descending from PRD-010) specifies the knowledge-record
entity surface: assumptions, decisions, questions, constraints — the typed, citable
home for the truth that *shapes* work without being work. The spec is sliced in three
(the pinned cut): **Slice A (this slice) — the standalone entity surface**; Slice B —
the relation seam (FR-005, → IMP-050); Slice C — supersession (FR-006, → IMP-051).

This slice stands up the four kinds as **one `knowledge_record` entity discriminated
by `record_kind`** over the shared kind-blind engine — their lifecycles, typed facets,
shared evidence, prefix→kind read resolution, the never-`Workable` priority partition,
and the `doctrine knowledge` CLI. **No relations, no supersession, no gating** — it
ships alone with zero cross-kind dependency.

It is **scaffold reuse, not new engine**. The structural twin is the backlog
(`src/backlog.rs`, SPEC-015): one entity, a `record_kind` discriminator over several
engine `Kind`s, the three-layer tolerant parse, the edit-preserving transition. The
variation this surface adds — four lifecycle vocabularies and four facet shapes — is
**data keyed by `record_kind`, never a parallel implementation** (NF-001).

## 2. Current State (what this slice rides, unchanged)

- **`src/backlog.rs`** — the closest twin. `ItemKind` (`clap::ValueEnum` + kebab
  serde) selects per-kind `Kind` consts (each its own `dir`/`prefix`/`scaffold`); the
  three-layer parse `RawBacklogToml → validate → BacklogItem` maps seeded-`""`/`[]`
  optionals via `optional_enum`/`optional_text`; `backlog_scaffold(kind, ctx)` picks a
  template by `has_facet()`; `set_backlog_status` is the edit-preserving `toml_edit`
  transition. `BACKLOG_STATUSES` + `is_hidden` drive `--status` known-set and the
  `listing::retain` hide-set.
- **`src/rec.rs`** — the `DecisionRef` free-text label site (D8); status-less scaffold
  (not the model here — records are status-ful).
- **`src/adr.rs`** — the per-kind closed-enum + `*_STATUSES` const + drift-canary
  idiom (reused for the three facet value-enums).
- **`src/entity.rs`** — the kind-blind materialiser (`ScaffoldCtx`, `Kind`, `Fileset`,
  `materialise_fresh_prebuilt`, `scan_ids`). `Kind` is data, not a trait — the verb
  seam is intentionally unabstracted (variation is the kind table).
- **`src/integrity.rs`** — `KINDS` (the corpus-wide id table, a *referencing view*
  over each module's `Kind` const) + the **ordered** golden
  `kinds_table_covers_the_numbered_kinds` (a literal prefix pin: `[…,"RV","REC"]`).
- **`src/priority/partition.rs`** — `PARTITION` (per-kind `workable`/`terminal` sets,
  keyed by prefix) + the per-kind VT-1 drift canary reading each `*_STATUSES`.
- **`src/listing.rs`** — `CommonListArgs`, `validate_statuses`, `retain`,
  `render_columns`, `json_envelope`, `canonical_id` (the SPEC-013 list spine).
- **Wiring surfaces** — `.gitignore` blanket `.doctrine/*` + per-tree negation;
  `install/manifest.toml [dirs].create` (mem.pattern.install.authored-entity-wiring).

No code exists for this surface yet.

## 3. Forces & Constraints

- **NF-001 (REQ-245) — one entity, one schema + behaviour preservation.** Four kinds as
  one `record_kind`-discriminated entity; never parallel per-kind schemas; reuse the
  shared scaffold so the existing slice/ADR/spec/backlog/memory (and relation) suites
  stay green **unchanged** (the entity engine is shared machinery).
- **NF-002 (REQ-246) — disjointness + outbound-only.** No `record_kind` may collide
  with a backlog `item_kind`; every outbound relation is stored one-way with the reverse
  derived (ADR-004), the supersession pair the sole sanctioned typed carve-out.
- **NF-003 (REQ-247) — never actionable.** No record state is ever `Workable`; records
  never appear in `survey`/`next`; identity permanent; `record_kind` fixed at capture.
- **ADR-001** — module layering: `knowledge.rs` is a leaf/command module; it may be
  read by `integrity`/`partition` (a referencing view over its `Kind`/`*_STATUSES`
  consts) but imports no peer kind module.
- **Pure/imperative split** — no clock/disk in the pure render/validate/partition core;
  the date is injected by the shell (the `clock::today()` pattern).
- **Storage rule + F1** — structured data in TOML, prose in MD; every typed table
  precedes any `[[relation]]` array (trivially held — Slice A seeds no `[[relation]]`).
- **SPEC-019 pins** — D1 (thin-not-anaemic component), D2 (`record_kind`-keyed
  lifecycle + facets), D4 (capture takes the kind, read resolves it from the prefix),
  D5 (shared minimal evidence), D7 (never-`Workable`, gating deferred to IMP-047),
  D8 (DEC dual-namespacing).

## 4. Decisions

SPEC-019 D1–D8 are inherited verbatim. This slice closes its remaining open questions
and adds the implementation-shaping local decisions (L-series):

- **L1 — status vocabulary is data, not a typed enum.** Each kind's status set is a
  `&'static [&'static str]` const (`ASSUMPTION_STATUSES`, …) with a `record_kind →
  &[&str]` lookup; `status` is stored and validated as a `String`. Rationale: the
  `status <ID> <state>` verb resolves `record_kind` from the id prefix at **runtime**,
  so clap cannot bind a typed `ValueEnum` across the four kinds — `<state>` is a string
  at the boundary regardless. A typed enum would add 4× boilerplate (enum + `as_str` +
  known-set + dispatch) for compile-time typo-catching the drift canary already covers.
  Matches the spec's "the kind table carries the per-kind status set" and "variation is
  data." (Closes the foundational modeling question.)
- **L2 — the facet is a typed `enum` over four per-kind structs.** `RecordFacet`
  carries one variant per kind; this **enforces structurally** that `confidence` is
  assumption-only (a constraint cannot hold one). The closed facet value-enums
  (`Confidence`, `Basis`, `ConstraintSource`) stay typed with drift canaries. Typed
  where types pay (heterogeneous fields, the assumption-only invariant); stringly only
  for the uniform status word-list (L1).
- **L3 (OQ2) — `DecisionRef` stays `Unvalidated` free-text.** The `decision_ref` label
  carries **external 3-part** forgettable cites (`DEC-005-C`), which are not doctrine
  entities; validating them against the new numbered DEC kind would reject live data.
  No behaviour change to the label — D8 work here is **comment/example disambiguation
  only** (§10).
- **L4 (A1) — naming accepted as SPEC-019 proposes.** `doctrine knowledge` namespace;
  `record-NNN.{toml,md}` fileset + `NNN-slug` symlink; `.doctrine/knowledge/<kind>/`
  trees; `record_kind` discriminator.
- **L5 (KINDS insertion) — append at end, after `REC`.** The ordered golden becomes
  `[…,"RV","REC","ASM","DEC","QUE","CON"]`, preserving every existing position (zero
  churn to other kinds' goldens). Relations are out of scope, so `RELATION_RULES` enum
  ordering is untouched here (note for Slice B: the `RelationLabel`/source-group enum
  ordering is a *separate* order from KINDS).
- **L6 — capture seeds an empty facet; no per-kind capture flags in v1.** `knowledge
  new <kind> [title]` seeds the default status + an empty `[facet]`/`[evidence]`; the
  body is filled by hand-editing the toml (the backlog risk-facet precedent). Every
  facet field is therefore optional.
- **L7 — `outbound_for` gets a four-prefix empty arm (the total-dispatch fix).**
  Admitting ASM/DEC/QUE/CON to `KINDS` without an `outbound_for` arm drops them through
  to `relation_graph.rs:65`'s `debug_assert!(false, "unrouted KINDS prefix")` — so once
  any record exists, every **debug-build** graph scan (`inspect`/`slice show`/
  `build_relation_graph`) panics (release is benign-empty). Slice A adds
  `"ASM" | "DEC" | "QUE" | "CON" => Ok(Vec::new())`, mirroring the existing
  `"REQ" => Ok(Vec::new())` arm (a kind that authors no outbound relations). This is
  **routing, not relations** — no rules, no labels, no reader; Slice B replaces the
  empty arm with the real `knowledge::relation_edges` accessor. (Found by the internal
  adversarial pass, §14 F-A1; it refutes the "touches no relation code" simplification
  and corrects the scope/spec claim that `sources_match_shipped_accessors` — which is
  label-keyed, not KINDS-keyed — is edited by Slice A.)
- **L8 — records are valid `reviews` targets the moment they join KINDS (allowed).**
  The shipped `reviews` label is `TargetSpec::AnyNumbered` (`relation.rs:354`), and
  `review new --target REF` validates via `ensure_ref_resolves` (`review.rs`/
  `integrity.rs:319`). So admitting ASM/DEC/QUE/CON to `KINDS` makes
  `doctrine review new --target ASM-001` resolve, and `inspect` surfaces the inbound
  `reviews` edge on the record — an **inbound, RV-authored** edge, free, with no Slice-A
  code beyond the KINDS rows. **Decision: allow it** — reviewing an assumption/decision
  is sensible; the alternative (special-casing `AnyNumbered` to exclude records) is more
  code, less general, and against "variation is data." It does **not** touch NF-003: the
  edge is the review's outbound, not a record dependency, so no record becomes workable.
  This is *not* the Slice-B relation seam (records still author no outbound relations) —
  it is RV's pre-existing reach extending to a new target kind. (Codex inquisition
  Charge 1, §14 C-1.)

## 5. Module & Types

New module **`src/knowledge.rs`** — the `backlog.rs` structural twin, riding
`entity`/`listing`/`tomlfmt`/`install::asset_text`.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RecordKind { Assumption, Decision, Question, Constraint }

pub(crate) const ASSUMPTION_KIND: Kind = Kind {
  dir: ".doctrine/knowledge/assumption", prefix: "ASM",
  scaffold: |c| record_scaffold(RecordKind::Assumption, c) };
// DECISION_KIND "DEC" · QUESTION_KIND "QUE" · CONSTRAINT_KIND "CON" — same shape.
```

`RecordKind` carries (read off the `Kind`, single source): `kind()`, `prefix()`,
`as_str()` (kebab), `from_prefix()`, `ALL`, `default_status()` (the seed).

**Status — data-driven (L1); hide-set distinct from the priority partition:**

```rust
pub(crate) const ASSUMPTION_STATUSES: &[&str] =
  &["held","testing","validated","invalidated","obsolete"];
pub(crate) const DECISION_STATUSES:   &[&str] = &["proposed","accepted","rejected","superseded"];
pub(crate) const QUESTION_STATUSES:   &[&str] = &["open","answered","obsolete"];
pub(crate) const CONSTRAINT_STATUSES: &[&str] = &["active","waived","superseded","retired"];

// default-list HIDE-set (settled states only) — NOT the full vocab:
const ASSUMPTION_HIDDEN: &[&str] = &["validated","invalidated","obsolete"];
const DECISION_HIDDEN:   &[&str] = &["rejected","superseded"];   // `accepted` stays visible
const QUESTION_HIDDEN:   &[&str] = &["answered","obsolete"];
const CONSTRAINT_HIDDEN: &[&str] = &["waived","superseded","retired"];

fn statuses(k: RecordKind) -> &'static [&'static str];   // vocab + known-set
fn is_hidden(k: RecordKind, status: &str) -> bool;        // reads *_HIDDEN → listing::retain
```

The `*_STATUSES` consts are `pub(crate)` (read by `partition.rs` canaries). The seed
is each vocab's first element (`held`/`proposed`/`open`/`active`).

**One entity, typed facet enum (L2):**

```rust
pub(crate) struct KnowledgeRecord {
  id: u32, slug: String, title: String,
  record_kind: RecordKind,
  status: String,                  // validated against statuses(kind) at the seam
  created: String, updated: String, tags: Vec<String>,
  facet: RecordFacet,
  evidence: Evidence,
}
enum RecordFacet { Assumption(AssumptionFacet), Decision(DecisionFacet),
                   Question(QuestionFacet), Constraint(ConstraintFacet) }
struct Evidence { supports: Vec<String>, contradicts: Vec<String>, notes: Vec<String> }
// closed facet value-enums (as_str + known-set + drift canary; "" -> None optional):
enum Confidence { Low, Medium, High }
enum Basis { Observation, PriorArt, DesignInference, ExternalSource, OperatorJudgement }
enum ConstraintSource { Canon, Adr, External, Technical, Legal, Compatibility, Operator }
```

**Three-layer parse** (existing idiom): `RawRecordToml` reads tolerantly — `status`
stays `String`; `[facet]` is read as **one kind-blind superset `RawFacet`** (every
field across all kinds, `#[serde(default)]`); `[evidence]` defaults empty. `validate`
dispatches on `record_kind` to build the right typed `RecordFacet` variant, mapping
`""`/`[]`→absent. **Kind-blind read, kind-aware validate** — one materialiser.

**On-disk order (F1):** template emits top-level meta → `[facet]` → `[evidence]`. Slice
A seeds **no** `[[relation]]` block and **no** typed `[relationships]` supersession pair
(B/C add those). `KnowledgeRecord` carries no `tier1` field yet (Slice B adds the
reader). Records are **status-ful** → scanned via the standard `meta::Meta` path.

**On-disk keys + meta round-trip (F-A3).** Top-level `id`/`slug`/`title`/`status`/
`created`/`updated`/`tags` round-trip into the strict `meta::Meta`; `record_kind` is a
top-level kebab key (the stored discriminator, also implied by the tree dir, stored for
one-read). `meta::read_meta` (used by `integrity` and `status_and_title_for`'s common
arm) tolerates the extra `record_kind`/`[facet]`/`[evidence]` keys — serde ignores
unknowns (proven by `adr.rs`'s `…relationships_are_preserved_and_ignored_by_meta`).

## 6. CLI Surface (`doctrine knowledge`)

Rides SPEC-013's `<kind> <verb>` grammar and the kind-blind listing spine; main.rs
subcommand + thin forwarders in `knowledge.rs`.

| verb | shape | behaviour |
|---|---|---|
| `new <record_kind> [title] [--slug]` | mirrors `backlog new` | reserve id in the kind's namespace; seed default status, empty `[facet]`, empty `[evidence]` |
| `show <ID> [--format]` | prefix→kind resolve | reassemble identity/kind/status/facet/evidence (`Table`/`Json`) |
| `list [CommonListArgs]` | cross-kind (all 4 trees) | `--status` known-set = **union of the 4 vocabs**; `is_hidden(kind,status)` per item; `--all`/explicit `--status` reveal; canonical-id/JSON/columns |
| `status <ID> <state>` | prefix→kind resolve | validate `<state>` ∈ `statuses(kind)`, **refuse a foreign-kind state**; edit-preserving `toml_edit` writes `status`+`updated` (**no resolution coupling**) |

**Prefix→kind resolution (FR-004):** `resolve_ref(&str) -> (RecordKind, u32)` splits the
prefix, maps via `RecordKind::from_prefix`, parses `NNN`. Shared by `show`/`status`.

## 7. Priority Partition & Lifecycle (NF-003 / D7)

Two distinct notions, deliberately kept apart:

- **Lifecycle hide-set** (`is_hidden`, §5) — the *settled* states that drop from the
  default `list`; `held`/`proposed`/`open`/`active` stay visible. Drives
  `listing::retain`.
- **Priority partition** (`partition.rs`) — records are **never** `Workable`, so each
  kind's entry is `workable: &[]`, `terminal: <KIND>_STATUSES` (the **full** vocab):

```rust
KindPartition { prefix: "ASM", workable: &[], terminal: knowledge::ASSUMPTION_STATUSES },
// DEC / QUE / CON — identical shape.
```

Four VT-1 canaries: `vocab("ASM") == set(ASSUMPTION_STATUSES)` … (holds: `∅ ∪ full =
full`). This is the **positive all-`Terminal` declaration**, not REC's status-less
`None → Terminal` path. Direct gating (the `Gating` class + record→item dep edge) is
**IMP-047** — out of scope; interim gating is via a spawned backlog item.

## 8. KINDS / integrity / install / dispatch wiring

- **`integrity::KINDS`** — append four `KindRef { kind: &…_KIND, stem: "record",
  state_dir: None }` after `REC`. Update the ordered golden to
  `[…,"RV","REC","ASM","DEC","QUE","CON"]`; the stateful assertion stays `["SL","RV"]`.
- **`relation_graph::outbound_for`** (L7, F-A1) — add
  `"ASM" | "DEC" | "QUE" | "CON" => Ok(Vec::new())` so the prefix dispatch stays total
  and debug-safe. The *only* relation-layer touch in Slice A; Slice B swaps it for the
  real accessor. `sources_match_shipped_accessors` and `status_and_title_for` need **no**
  change (the former is label-keyed; records route through the latter's common `_` arm).
- **`.gitignore`** — add `!.doctrine/knowledge/` (one negation covers all four
  subtrees, the backlog precedent). Without it the tree is silently uncommittable.
- **`install/manifest.toml [dirs].create`** — add `.doctrine/knowledge` (per-kind
  subtrees mkdir on demand; parity/discoverability, the backlog precedent).

## 9. Facet Shapes (OQ1, review M6)

Type legend: **text** = `Option<String>`, `""`→`None`; **enum(…)** = closed typed
enum, `Option<T>`, `""`→`None`, drift-canaried; **list** = `Vec<String>`, `[]` default;
**date** = `Option<String>` ISO, `""`→`None`, *unvalidated* (the corpus-wide
`created`/`updated` convention — no typed `Date`). All fields seeded empty at capture
(L6).

**assumption (ASM):** `claim` text · `confidence` enum(low,medium,high) *[assumption-only]*
· `basis` enum(observation,prior-art,design-inference,external-source,operator-judgement)
· `validation_plan` text · `validated_by` text · `validated_on` date · `invalidated_by`
text · `invalidated_on` date.

**decision (DEC):** `context` text · `choice` text · `alternatives` **list** ·
`rationale` text · `consequences` **list** · `decided_by` text · `decided_on` date.

**question (QUE):** `question` text · `why_matters` text · `answer` text · `answered_by`
text · `answered_on` date.

**constraint (CON):** `statement` text · `source`
enum(canon,adr,external,technical,legal,compatibility,operator) · `applies_to` **list** ·
`waiver_reason` text · `waived_by` text · `waived_on` date.

**evidence (shared, all four):** `supports` list · `contradicts` list · `notes` list —
free-text citations (D5: a minimal citation structure, never queryable graph machinery
in v1).

**M6 resolutions:** plural fields (`alternatives`/`consequences`/`applies_to`) →
**list**; every `…_by` → **text attribution** (not a graph ref — kept out of the Slice-B
relation machinery); every `…_on` → **date** (unvalidated string). Three closed enums →
three drift canaries (`confidence`/`basis`/`source`).

## 10. D8 Disambiguation (no behaviour change — `DecisionRef` stays `Unvalidated`, L3)

The numbered DEC kind makes the 2-part `DEC-NNN` form read as a doctrine entity, so
every site that uses 2-part `DEC-` as an *external* forgettable cite is now misleading.
Disambiguate them all to the unmistakable 3-part `DEC-NNN-XX` external form:

- `src/rec.rs:318` comment — reword: a DEC *is* now a 2-part numbered kind, but
  `decision_ref` carries **external 3-part** forgettable cites (`DEC-005-C`), not
  entities → still carries free-text.
- `src/relation.rs:164` `TargetSpec::Unvalidated` doc — same reword (rationale shifts
  from "no kind in KINDS" to "3-part external cites are not entities").
- `src/main.rs:1537` `--decision` example `DEC-005` → `DEC-005-C`.
- **Fixtures (disambiguate — the spec + scope name these D8 targets; codex Charge 2).**
  `src/rec.rs:673` (`decision_ref="DEC-005"`) and `src/relation_graph.rs:1062`
  (`decision_ref="DEC-001"`), plus any sibling `DEC-001`/`DEC-005` *decision_ref* fixture
  values → 3-part external form, **updating the matching assertion in the same test** so
  it stays green. `DecisionRef` is `Unvalidated`, so the string is opaque to behaviour —
  this is a lockstep value+assertion edit, not an engine-behaviour change (NF-001 holds:
  the value swap is a *deliberate* clarity edit, not a forced fix to make a suite pass).
  Earlier draft pardoned these "to honour green-unchanged" — that was overcautious and
  contradicted the scope; superseded.
- **Sweep (F-A4)** — at execute, grep `DEC-0` across `install/templates/`, `doc/`, and
  `.claude/` for other 2-part `DEC-NNN` examples used as *external* cites that the new
  numbered kind now makes ambiguous; disambiguate to 3-part or note them. D8's named
  sites are necessary, not provably exhaustive.

## 11. Verification Alignment

- **Round-trip (VT, per kind):** a fully-populated `record-NNN.toml` (facet + evidence)
  survives toml→struct→toml.
- **Drift canaries:** 3 facet-enum (`confidence`/`basis`/`source`) + 4 partition VT-1 +
  the per-kind status known-set.
- **Optional seam:** `""`/`[]`→absent for the optional facet fields.
- **Scaffold:** 2 files + symlink per kind; F1 ordering. **Seed-status anti-drift
  (F-A2):** assert the scaffolded toml's `status` == `default_status(kind)` per kind
  (the seed lives in both the template literal and `default_status()`).
- **Read path:** prefix→kind resolution; foreign-kind-state **refuse** (FR-002/FR-004).
- **`outbound_for` total-dispatch (F-A1):** `outbound_for` returns `Ok(vec![])` (never
  panics) for each of ASM/DEC/QUE/CON — the regression guard for the empty arm.
- **Scan-side totality (F-A7, the L7 partner):** `build_relation_graph` scans **every**
  `KINDS` dir, so admitting the four rows makes each graph scan visit the record trees
  even on a record-less repo. It is benign only because `entity::scan_ids` returns
  `Ok(vec![])` on a missing dir (`entity.rs:195`, `NotFound → empty`) — the load-bearing
  guarantee behind "existing suites green unchanged" for the all-KINDS scan, symmetric to
  L7's outbound-side totality. Guard: a `build_relation_graph` over a fixture with the
  KINDS rows present but **no** record tree returns the pre-existing graph unchanged
  (regression tripwire if `scan_ids` is ever made strict).
- **RV→record reviews target (L8 / C-1):** `review new --target ASM-001` resolves
  (`ensure_ref_resolves`), and a cross-kind graph scan / `inspect` surfaces the inbound
  `reviews` edge on the record — confirming the free `AnyNumbered` surface is sound and
  records stay non-workable.
- **Decision `accepted` divergence (F-A5):** pin `is_hidden(Decision,"accepted")==false`
  (visible) **and** `status_class("DEC","accepted")==Terminal` (never workable) — the two
  concepts deliberately disagree on this state.
- **CLI:** black-box per-verb goldens (new/show/list/status) + the SPEC-013
  parse-conformance matrix row for `knowledge`; kind-relative `--status` known-set;
  hide-set behaviour (`--all`/explicit reveal); a cross-kind `--status` filter on a
  **shared** token (`obsolete`, `superseded`) returns items across kinds.
- **Disjointness (NF-002 / F-A6):** the four new prefixes collide with **no** existing
  corpus prefix (not just backlog — all of SL/ADR/POL/STD/PRD/SPEC/REQ/ISS/IMP/CHR/RSK/
  IDE/RV/REC); the new `KINDS` rows don't overlap any existing partition.
- **Behaviour preservation (NF-001):** slice/ADR/spec/backlog/memory/relation suites
  green **unchanged**.

## 12. Out of Scope / Deferred

- **Relation seam (FR-005)** — Slice B (→ IMP-050). No `RECORD` `RELATION_RULES` rows,
  no minted labels, no record `relation_edges` reader/accessor, no edges. **Slice A does
  land the *empty* four-prefix `outbound_for` arm** (§4 L7, §8) — routing only, no
  relations; B swaps it for the real accessor.
- **Supersession (FR-006)** — Slice C (→ IMP-051); IMP-006-gated.
- **Direct gating** — IMP-047 (the `Gating` priority class). Interim: all-`Terminal`
  inert, gating via a spawned backlog proxy.
- **Record↔record associative relations** (e.g. QUE↔ASM "the assumption I hold about
  this question", ASM→DEC "this belief shaped this decision") — **not covered by any
  current SPEC-019 label**; captured as **IMP-053** (a SPEC-019 amendment feeding
  Slice B).
- **Constraint owner / immutability-or-enforceability axis** — captured as **IDE-006**.
- **Guidance: DEC record vs ADR vs governance surface** — captured as **IDE-007**.
- **Memory↔record seam** — OQ-1 / PRD-010 OQ-006/007, v2.
- **Renaming external forgettable `DEC-NNN-XX` cites** — provenance, never renumbered.

## 13. Risks & Open Questions

- **R1 — ordered-golden churn.** `KINDS` and its golden are ordered; appending after
  `REC` (L5) is deliberate and minimal. Mitigated by the explicit golden update.
- **R2 — superset `RawFacet` laxity.** A kind-blind raw facet admits a stray
  foreign-kind key (ignored by the kind-aware `validate`). Accepted: tolerant-read
  behaviour, matching `RawBacklogToml`; the validated `RecordFacet` is fully typed.
- **R3 — behaviour preservation (NF-001).** The engine `Kind` is data, not a trait, so
  the four new kinds add table rows, not engine changes; the existing suites are the
  proof and stay green unchanged.
- **R4 — disjointness.** New prefixes ASM/DEC/QUE/CON must not collide with backlog
  ISS/IMP/CHR/RSK/IDE — they don't; enforced by the KINDS golden + a disjointness test.
- No open design questions remain; all SPEC-019 OQs for Slice A are closed (OQ1 §9,
  OQ2 §4 L3, A1 §4 L4, KINDS insertion §4 L5).
- **R5 — `outbound_for` total-dispatch (was the sharpest miss; now §4 L7 / §8 / §14
  F-A1).** Resolved by the empty four-prefix arm.

## 14. Internal Adversarial Pass

A hostile self-review before external challenge. Findings + dispositions:

- **F-A1 (MUST-FIX, integrated).** Adding KINDS rows without an `outbound_for` arm
  drops ASM/DEC/QUE/CON to `relation_graph.rs:65`'s `debug_assert!(false)` — every
  debug-build graph scan panics once a record exists. → §4 L7, §8 (empty arm), §11
  (total-dispatch guard). Also corrected the inherited scope/spec claim that
  `sources_match_shipped_accessors` is edited by Slice A — it is label-keyed and
  untouched; only `outbound_for` is.
- **F-A2 (integrated).** Seed status duplicated (template literal + `default_status()`)
  → §11 anti-drift test.
- **F-A3 (integrated).** On-disk `record_kind` key + strict-`meta` tolerance pinned → §5.
- **F-A4 (integrated).** D8's named sites aren't provably exhaustive → §10 execute-time
  `DEC-0` sweep.
- **F-A5 (integrated).** `accepted` decision is list-visible yet partition-Terminal —
  deliberate, surprising → §11 dual assertion.
- **F-A6 (integrated).** Disjointness must cover all corpus prefixes, not just backlog
  → §11.
- **Checked, no change:** `status_and_title_for` routes records through its common `_`
  arm (status-ful, top-level status+title) — safe. `read_block`/`title_for` are generic
  TOML reads (no per-kind dispatch assert). ADR-001 layering holds (knowledge.rs is a
  leaf; integrity/partition/relation_graph reference its consts, the existing pattern).
  The superset `RawFacet` laxity is the sanctioned tolerant-read tier (R2).

### External adversarial pass (codex / GPT-5.5)

Ran via the inquisition brief (`handover.md`). It **confirmed** the load-bearing
claims: `status_and_title_for` safe through the common arm, `sources_match_shipped_
accessors` label-keyed (untouched by A), `DecisionRef` forward-edge behaviourally
`Unvalidated`, and **no second KINDS-prefix dispatch-panic** beyond `outbound_for`.
Four charges, all integrated:

- **C-1 (must-fix).** Blast-radius understatement: records become valid `reviews`
  targets (`AnyNumbered`) on KINDS admission. → §4 L8 (acknowledge + allow), §11 guard.
- **C-2 (should-fix).** Self-contradiction — scope names the D8 fixtures as targets,
  §10 pardoned them. → §10 now disambiguates them (lockstep value+assertion edit).
- **C-3 (should-fix).** Self-contradiction — §12 said "no `outbound_for` arm" vs L7. →
  §12 corrected.
- **C-4 (nit).** REQ miscitation — behaviour-preservation is NF-001/REQ-245, not
  NF-002 (disjointness). → §3/§11/§13 citations corrected.

### Second Opus adversarial pass (variety reviewer)

A third hostile pass, source-verified rather than design-trusted. **Verdict: clean —
lock.** Every load-bearing claim re-confirmed against source: `outbound_for` empty-arm
necessity (`relation_graph.rs:65` `debug_assert`), the KINDS golden append, `meta::Meta`
carrying no `deny_unknown_fields` (F-A3 holds), `status_and_title_for`'s common arm
(records status-ful, top-level), `sources_match_shipped_accessors` label-keyed (untouched
by A), `reviews`/`AnyNumbered` (`relation.rs:355`, L8), the partition empty-`workable`
canary, and **facets matching SPEC-019 prose exactly** (validated/invalidated/decided/
answered by-and-on, waiver reason + waived by-and-on).

- **F-A7 (nit, integrated).** Named the scan-side total-dispatch guarantee — the L7
  partner. `build_relation_graph` scans all KINDS dirs; admitting the rows is benign only
  because `scan_ids` tolerates a missing dir (`entity.rs:195`). → §11 guard.
- **Noted for `/plan` (no design change):** gitignore/install wiring (§8) need not land
  before P1–P3 (tempdir tests don't touch the real tree) but should not lag the first
  real-record use; KINDS rows and the `outbound_for` arm are coupled (must co-land — a
  KINDS row without the arm panics every debug graph scan), so they belong in one phase.
</content>
</invoke>
