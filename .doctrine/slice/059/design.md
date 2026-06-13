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

- **NF-001 (REQ-245) — one entity, one schema.** Four kinds as one
  `record_kind`-discriminated entity; never parallel per-kind schemas; reuse the shared
  scaffold so existing suites stay green.
- **NF-002 (REQ-246) — disjointness + behaviour preservation.** No `record_kind` may
  collide with a backlog `item_kind`; the slice/ADR/spec/backlog/memory/relation suites
  stay green **unchanged** (the entity engine is shared machinery).
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
  carries **external 3-part** the external decision register cites (`DEC-005-C`), which are not doctrine
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

## 8. KINDS / integrity / install wiring

- **`integrity::KINDS`** — append four `KindRef { kind: &…_KIND, stem: "record",
  state_dir: None }` after `REC`. Update the ordered golden to
  `[…,"RV","REC","ASM","DEC","QUE","CON"]`; the stateful assertion stays `["SL","RV"]`.
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

## 10. D8 Disambiguation (comments/example only — no behaviour change)

`DecisionRef` stays `Unvalidated` (L3). The numbered DEC kind makes some prose stale:

- `src/rec.rs:318` comment — reword: a DEC *is* now a 2-part numbered kind, but
  `decision_ref` carries **external 3-part** the external decision register cites (`DEC-005-C`), not
  entities → still carries free-text.
- `src/relation.rs:164` `TargetSpec::Unvalidated` doc — same reword (rationale shifts
  from "no kind in KINDS" to "3-part external cites are not entities").
- `src/main.rs:1537` `--decision` example `DEC-005` → `DEC-005-C`.
- **Fixtures** (`relation_graph.rs`/`rec.rs`, `decision_ref="DEC-001"`/`"DEC-005-C"`):
  **left untouched.** They are green regardless (`Unvalidated` carries any string), and
  NF-002 requires existing suites green-*unchanged*. The cosmetic 2-part→3-part swap is
  skipped to honour behaviour-preservation.

## 11. Verification Alignment

- **Round-trip (VT, per kind):** a fully-populated `record-NNN.toml` (facet + evidence)
  survives toml→struct→toml.
- **Drift canaries:** 3 facet-enum (`confidence`/`basis`/`source`) + 4 partition VT-1 +
  the per-kind status known-set.
- **Optional seam:** `""`/`[]`→absent for the optional facet fields.
- **Scaffold:** 2 files + symlink per kind; correct default-status seed; F1 ordering.
- **Read path:** prefix→kind resolution; foreign-kind-state **refuse** (FR-002/FR-004).
- **CLI:** black-box per-verb goldens (new/show/list/status) + the SPEC-013
  parse-conformance matrix row for `knowledge`; kind-relative `--status` known-set;
  hide-set behaviour (`--all`/explicit reveal).
- **Disjointness (NF-002):** no prefix collision with backlog; the two `KINDS`
  partitions don't overlap.
- **Behaviour preservation (NF-002):** slice/ADR/spec/backlog/memory/relation suites
  green **unchanged**.

## 12. Out of Scope / Deferred

- **Relation seam (FR-005)** — Slice B (→ IMP-050). No `RECORD` `RELATION_RULES` rows,
  no minted labels, no `outbound_for` arm, no record `relation_edges` reader.
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
- **Renaming external the external decision register `DEC-NNN-XX` cites** — provenance, never renumbered.

## 13. Risks & Open Questions

- **R1 — ordered-golden churn.** `KINDS` and its golden are ordered; appending after
  `REC` (L5) is deliberate and minimal. Mitigated by the explicit golden update.
- **R2 — superset `RawFacet` laxity.** A kind-blind raw facet admits a stray
  foreign-kind key (ignored by the kind-aware `validate`). Accepted: tolerant-read
  behaviour, matching `RawBacklogToml`; the validated `RecordFacet` is fully typed.
- **R3 — behaviour preservation (NF-002).** The engine `Kind` is data, not a trait, so
  the four new kinds add table rows, not engine changes; the existing suites are the
  proof and stay green unchanged.
- **R4 — disjointness.** New prefixes ASM/DEC/QUE/CON must not collide with backlog
  ISS/IMP/CHR/RSK/IDE — they don't; enforced by the KINDS golden + a disjointness test.
- No open design questions remain; all SPEC-019 OQs for Slice A are closed (OQ1 §9,
  OQ2 §4 L3, A1 §4 L4, KINDS insertion §4 L5).
</content>
</invoke>
