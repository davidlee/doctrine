# Design SL-026: lazyspec read-only projection

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Emit a read-only JSON projection of doctrine's entities in lazyspec's vocabulary,
so a lazyspec doctrine backend (piece 4, `../lazyspec`,
https://github.com/jkaloger/lazyspec) can render doctrine as a front-end without
either side learning the other's internals. This slice owns the **wire format** and
its **producer** — doctrine-side pieces 2 + 3 of the integration (research brief at
`../lazyspec/research/lazyspec-doctrine-integration-brief.md`; decisions in
`mem.thread.lazyspec.frontend-integration`).

The brief established the boundary: lazyspec consumes a `{meta, entities[], types[]}`
JSON envelope shaped to its `DocMeta` model; every entity carries
`validate_ignore: true` (doctrine owns validation) and emitted types are
non-singleton; richer doctrine edges flatten to lazyspec's four `RelationType`s;
composed-spec bodies are assembled inline; requirements are **not** standalone nodes.

## 2. Current State

doctrine already emits JSON: SL-025 (done, closed) shipped a shared spine —
`listing::Format {Table, Json}` (`src/listing.rs:39`), `json_envelope<T: Serialize>`
→ `{kind, rows}` (`:304`), `canonical_id(prefix, id)` → `{PREFIX}-{NNN}` (`:35`), and
per-kind `show_json`. **But that JSON is doctrine-native** (toml-as-data + doctrine
fields), keyed per-command, not a cross-kind aggregate in lazyspec's shape. No
projection to an external consumer's model exists.

Read APIs to reuse (no new read logic): spec composition — `spec::read_members`
(`:458`), `read_interactions` (`:481`), the **pure** `render()` (`:337`, assembles
spec+prose+members+interactions), `requirement::load` (`requirement.rs:222`); plus
adr/slice/backlog loaders and `state::PhaseRollup` (`slice.rs:389` consumes it).

Entity inventory (prefixes confirmed from `Kind` consts): slice `SL`; spec `PRD`
(product) + `SPEC` (tech); adr `ADR`; backlog one kind, five item prefixes
`ISS`/`IMP`/`CHR`/`RSK`/`IDE`; requirement `REQ`; **plan is not a reserved entity** —
`PLAN_KIND` shares `SL` and lives inside `slice/nnn/`.

## 3. Forces & Constraints

- **ADR-001 (module layering, leaf ← engine ← command).** `project` is a pure
  *function* (no I/O), but it reads entity structs (`Spec`, `Adr`, backlog `Item`,
  slice meta) that live in **command-layer** modules — so it sits in the export
  command module *above* them (forward edges, no cycle; nothing lower imports it).
  Purity ≠ leaf-ness: the property here is side-effect-freedom, not bottom-of-stack.
  disk/clock/git live only in the impure shell.
- **Pure/imperative split.** No clock/rng/git/disk in the pure layer — `now` and
  loaded data are injected (the date/uid pattern).
- **ADR-004 (relations outbound-only; reciprocity derived).** Emit only outbound
  edges; lazyspec's `build_links` derives the reverse. The models agree — no inbound
  synthesis.
- **Ride SL-025, don't fork it.** Reuse `canonical_id` + the read APIs. The Brief is
  *not* the `{kind, rows}` envelope and *not* a `Format` variant (a cross-kind
  aggregate has no per-command home) — it is its own shape, serialized via plain
  `serde_json`. That is not a parallel renderer: the duplication avoided is
  read/compose logic, not `serde`.
- **No masquerade.** The command is named for its target (`export lazyspec`) so the
  output never reads as canonical/native doctrine JSON.
- **lazyspec graph follows `Implements` only** (brief §6) — shapes the edge mapping.
- **Repo clippy denials** — BTree not Hash; no indexing-slicing; `expect`+reason not
  bare `allow`; the string-assembly rules. (Memory cluster `mem.pattern.lint.*`.)
- **Depends on SL-028 (lands first).** SL-028 replaces the slice lifecycle
  vocabulary with a 10-state FSM
  (`proposed→design→plan→ready→started→review→audit→reconcile→done` + `abandoned`);
  the slice status mapping (§5.3) is built on **that** FSM, not the legacy 6-state
  set. doctrine has **no typed slice-dependency edge** — the `[relationships]`
  table carries only `specs`/`requirements`/`supersedes` (`slice.rs:572`) — so this
  dependency is recorded in prose (here + the scope Assumptions), not as a stored
  edge. Slice status is a free `String` with tolerated out-of-vocab drift
  (`slice.rs:349`/`:368`), so the map must be **total** (a default arm), never a
  partial allowlist.
- **Reuse SL-027's fixture builder (no parallel impl).** SL-027 (done) DRY'd the
  backlog test-fixture builders into `write_fixture`/`Fixture` — currently private
  to `backlog.rs`'s `#[cfg(test)] mod tests`. The golden corpus (§9) must ride it,
  not re-roll backlog TOML (which re-opens ISS-001, just closed).

## 4. Guiding Principles

Read-only, lossy-by-design v1 (brief): the projection serves a *viewer*, not a
round-trip. Where doctrine's model is richer than `DocMeta`, flatten or inline — do
not extend `DocMeta`, do not couple the schemas. Keep the leaf pure and the wire
shape pinned.

## 5. Proposed Design

### 5.1 System Model

```
doctrine export lazyspec  (command, impure shell)
  │  load corpus via existing readers (slices, specs+members+reqs, adrs, backlog,
  │     per-slice plan.md + PhaseRollup)
  │  inject now (RFC3339), version (CARGO_PKG_VERSION), project (root basename)
  ▼
lazyspec::project(corpus, now, version) -> Brief   (pure fn, command layer — src/lazyspec.rs)
  ▼
serde_json::to_string_pretty(&brief) -> stdout
```

`Corpus` is a plain pre-loaded data struct (Vecs of loaded entities + each slice's
optional `(plan_body, PhaseRollup)`). `project` is a total, side-effect-free function
over it — deterministic given `(corpus, now, version)`, which is what makes the
golden test deterministic (§9).

**Reuse needs visibility widening (code impact).** `spec::render`, `read_members`,
`read_interactions` are private `fn` today; reuse from the export module makes them
`pub(crate)`. That is an edit to `spec.rs` under the **behaviour-preservation gate** —
existing spec suites must stay green unchanged.

### 5.2 Interfaces & Contracts

CLI: `doctrine export lazyspec [--path <root>]` → Brief JSON on stdout, exit 0.
New top-level `Export` command enum with a `Lazyspec` variant (`src/main.rs`).

Wire structs (`src/lazyspec.rs`), mirroring brief §3 — two reserved-word renames:

```rust
#[derive(Serialize)]
struct Brief { meta: BriefMeta, entities: Vec<Entity>, types: Vec<TypeDef> }

#[derive(Serialize)]
struct BriefMeta { project: String, generated_at: String, doctrine_version: String }

#[derive(Serialize)]
struct Entity {
    id: String,                                    // canonical_id, or synthetic for plan
    kind: String,                                  // lazyspec type name
    title: String,
    status: String,                                // wire string ∈ lazyspec's 7 (§5.3)
    author: String,                                // "" where doctrine has none
    date: String,                                  // YYYY-MM-DD only (lazyspec parses %Y-%m-%d — NOT a datetime)
    tags: Vec<String>,
    related: Vec<Relation>,                        // outbound only
    body: String,                                  // assembled inline
    #[serde(rename = "virtual")] is_virtual: bool, // `virtual` reserved
    validate_ignore: bool,                         // always true
}

#[derive(Serialize)]
struct Relation { #[serde(rename = "type")] rel_type: String, target: String }  // `type` reserved

#[derive(Serialize)]
struct TypeDef { name: String, plural: String, dir: String, prefix: String, icon: String }
```

`rel_type` ∈ `{"implements","supersedes","blocks","related-to"}` only — the four
lazyspec `RelationType` strings; nothing else may appear (verified against
`RelationType::ALL_STRS`/`FromStr`, lazyspec `document.rs:128`).

### 5.3 Data, State & Ownership

**Type set & node mapping** (the contract this slice owns):

| lazyspec type | doctrine source | prefix | virtual | body | graph role |
|---|---|---|---|---|---|
| slice | slice | SL | no | scope `.md` | root |
| product-spec | spec (PRD) | PRD | yes | `render()` (reqs inline) | root |
| tech-spec | spec (SPEC) | SPEC | yes | `render()` (reqs inline) | child of PRD |
| adr | adr | ADR | no | adr `.md` | flat |
| issue | backlog ISS | ISS | no | item `.md` | flat / by axis |
| improvement | backlog IMP | IMP | no | item `.md` | flat / by axis |
| chore | backlog CHR | CHR | no | item `.md` | flat / by axis |
| risk | backlog RSK | RSK | no | item `.md` | flat / by axis |
| idea | backlog IDE | IDE | no | item `.md` | flat / by axis |
| plan | slice-child artifact | PLAN *(synthetic id `PLAN-NNN`)* | no | `plan.md` | child of slice |

Requirements (`REQ`) are **not** nodes — inlined in spec bodies via `render()` as
`FR-`/`NF-` labelled entries.

**Edge mapping** (doctrine → lazyspec four; outbound only):

| doctrine edge | lazyspec `type` | graph-visible |
|---|---|---|
| spec `descends_from` (tech → PRD) | implements | ✅ (D2 — lineage DAG) |
| spec `parent` (tech decomposition) | implements | ✅ |
| plan → slice (synthetic) | implements | ✅ |
| spec `interactions` (tech ↔ tech) | related-to | panel |
| adr `supersedes` | supersedes | panel |
| slice `supersedes` (when populated) | supersedes | panel |
| backlog `relationships.slices` (→ slice) | related-to | panel |
| backlog `relationships.specs` (→ spec) | related-to | panel |
| backlog `relationships.drift` (→ drift ref) | related-to | panel |

Backlog `Relationships` (`backlog.rs:374`) has exactly these three outbound axes,
all reference/association links — **none maps to `implements` or `blocks`** (no
backlog axis carries a dependency/sequence edge).

**Status mapping** — doctrine status → the **wire string**, ∈ lazyspec's 7:
`draft`/`review`/`accepted`/`in-progress`/`complete`/`rejected`/`superseded`
(verified against `Status` serde, lazyspec `document.rs:89` —
`#[serde(rename_all = "lowercase")]` + `InProgress → "in-progress"`). The mapping
is **TOTAL**: slice status is a free `String` with tolerated out-of-vocab drift
(`slice.rs:349`/`:368`), so an unknown/drifted status hits the default arm, never
panics or invents a string.

- slice — the **SL-028 FSM** (SL-028 lands first; replaces the legacy 6-state set):
  `{proposed→draft, design→draft, plan→draft, ready→accepted, started→in-progress,
  review→in-progress, audit→in-progress, reconcile→in-progress, done→complete,
  abandoned→rejected}`. **Default (drift / unknown) → `draft`.**
- spec `{draft→draft, active→accepted, deprecated→superseded, superseded→superseded}`
- adr `{proposed→review, accepted→accepted, rejected→rejected, superseded→superseded, deprecated→superseded}`
- backlog `{open→draft, triaged→review, started→in-progress, resolved→complete, closed→complete}`
- plan ← `PhaseRollup`: `completed==total && total>0 → complete`; `completed>0 → in-progress`; else `draft`.

`meta`: `project` = root dir basename; `generated_at` = injected `now` (RFC3339 —
doctrine's own meta field, not NaiveDate-parsed); `doctrine_version` =
`CARGO_PKG_VERSION`. `types[]` built from `Kind` consts (prefix, dir) + an assigned
icon + plural — **except** the synthetic `plan` type, which has no `Kind` const and
is therefore **hand-authored** (prefix `PLAN`, dir, icon, plural).

**Plan node `date`** = the owning slice's `updated` (injected as data; never read in
the pure layer). `DocMeta.date` is mandatory and must parse `%Y-%m-%d`
(`document.rs:11`), so the synthetic node can never ship an empty date.

**Entity & type ordering.** `project` sorts `entities[]` by canonical id and
`types[]` by name before serialization — disk-walk order is not stable, so this is
what makes §5.4's idempotence claim true and the golden file robust against
fixture-order churn.

### 5.4 Lifecycle, Operations & Dynamics

Single-shot, stateless, idempotent given a fixed tree + `now`. Cold path: lazyspec
invokes the command once and caches the body (its concern, brief §7). No reload,
no mutation, no side effects beyond stdout.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** every entity emits `validate_ignore: true`.
- **INV-2** `rel_type` is one of exactly four strings.
- **INV-3** emitted `types[]` are non-singleton (no `singleton` hint) so lazyspec's
  `TypeConstraintChecker` stays satisfied (brief §6).
- **INV-4** no `REQ` appears as an entity; every membered `REQ` appears in its
  spec's body.
- **INV-5** plan ids are the only synthetic ids; shape `PLAN-NNN` where `NNN` is the
  owning slice number; grammar-conformant (`{PREFIX}-{NNN}`) yet collision-free (no
  real `PLAN` reservation exists), so it cannot clash with a real `canonical_id`.
- **Edge cases:** slice with no plan → no plan node; spec with no members → body is
  prose-only, still virtual; backlog item kinds map 1:1 to five types; empty corpus →
  `entities: []`, `types[]` still full (manifest is static).
- **INV-6** every emitted `status` is one of the seven verified wire strings; an
  out-of-vocab doctrine status takes the per-kind default (slice → `draft`).
- **INV-7** every emitted `date` is `YYYY-MM-DD` (never a datetime) — incl. the
  synthetic plan node (owning slice's `updated`).
- **Dangling edges:** a `related[].target` outside the emitted corpus is dropped
  silently (lazyspec's `BrokenLinkRule` is suppressed by `validate_ignore`, brief
  §6); v1 accepts this. Option: filter to in-corpus targets at projection time.
- **Assumption:** lazyspec degrades on a write-refusing backend except the editor
  key — that gating is piece-4 (`../lazyspec`), not this slice.

## 6. Open Questions & Unknowns

- **OQ-1** Icon assignment per type is cosmetic; pick stable glyphs, not load-bearing.
- **OQ-2** `types[].dir` is nominal (lazyspec materializes bodies into its own cache);
  emit a sensible per-kind path string, not doctrine's real on-disk layout.
- **OQ-3 (RESOLVED — was adversarial F1).** Wire strings verified against lazyspec
  source, no fork needed: `Status` serde (`document.rs:89`) =
  `#[serde(rename_all = "lowercase")]` + `InProgress → "in-progress"` →
  `draft/review/accepted/in-progress/complete/rejected/superseded`; `RelationType`
  (`document.rs:128`) `ALL_STRS`/`FromStr` →
  `implements/supersedes/blocks/related-to`; `DocMeta.date` parses strictly
  `%Y-%m-%d` (`document.rs:11`) — date-only, not a datetime. The golden file encodes
  these; no string is guessed.
- **OQ-4 (adversarial F4)** Body for slice/adr/backlog: raw prose-tier `.md` (simplest;
  may be empty, drops structured TOML like acceptance_criteria/c4_level/risk facet) vs
  a both-tier synthesis (preserves unmapped data per the brief's "exotic data in body").
  Specs already get both tiers via `render()`. Decide per-kind in planning.
- **OQ-5 (RESOLVED — was adversarial F7).** Backlog `Relationships` (`backlog.rs:374`)
  has exactly three outbound axes — `slices`, `specs`, `drift` — all
  reference/association links; **all three → `related-to`**. No axis maps to
  `implements` or `blocks` (none is a dependency/sequence edge).
- **OQ-6** `meta.project` = root dir basename is non-canonical (differs across clones);
  cosmetic for lazyspec, accept for v1.

## 7. Decisions, Rationale & Alternatives

- **D1 — Aggregate command, not a `Format` variant or `json_envelope` reuse.** The
  Brief is cross-kind in lazyspec's vocabulary; the spine's per-command `{kind, rows}`
  contract has no home for it. Naming it `export lazyspec` prevents any masquerade as
  native output. *Alt rejected:* `Format::Lazyspec` per-command — would emit one kind
  at a time, wrong shape.
- **D2 — `descends_from` → implements (graph-visible).** doctrine's primary spec
  lineage renders as a DAG — the chief reason to use lazyspec for specs. Cost: the
  lazyspec edge label reads "implements", the overclaim doctrine rejected for the
  field name (`mem.concept.spec.descent-descends-from`). Accepted because it is a
  **display-only** graph-vocabulary label; doctrine's stored `descends_from` is
  untouched. *Alt rejected:* → related-to — honest label, but lineage vanishes from
  the graph (panel-only), gutting the value.
- **D3 — Plan as synthetic child node (`PLAN-NNN`).** Delivers the pictured
  slice→plan graph child though plan is not a reserved entity. The synthetic id is
  projection-only, never persisted — a bounded departure from "doctrine owns ids".
  Uses a grammar-conformant `PLAN-NNN` (own lazyspec type/prefix) rather than a
  `~`-suffixed form, so lazyspec's `{PREFIX}-{NNN}` id/type inference is not tripped
  (adversarial F3). *Alt rejected:* fold plan into slice body — simpler, loses the
  child node.
- **D4 — Backlog → five lazyspec types** (per item_kind prefix), since lazyspec keys
  a type by one prefix. *Alt rejected:* one "backlog" type — ambiguous prefix.
- **D5 — Spec → two types** (product-spec/PRD, tech-spec/SPEC) — preserves doctrine's
  subtype split; both virtual, reqs inline.
- **D6 — Emit outbound edges only** (ADR-004) — lazyspec derives reciprocity.

## 8. Risks & Mitigations

- **R1 — Wire schema drift** silently breaks the lazyspec backend. *Mitigate:*
  golden-file + every-surface conformance tests pin the shape; schema is
  version-fragile (`mem.pattern.parse.toml-error-classification-fragile`).
- **R2 — Envelope-parity ≠ surface-parity** (`mem.pattern.testing.conformance-asserts-surface-not-just-envelope`,
  the SL-025 audit miss). *Mitigate:* table-driven conformance over kinds **and**
  fields, asserting each surface (`virtual`, `validate_ignore`, edge vocab, renames,
  synthetic id, req-absence-but-body-presence).
- **R3 — dead_code** if phase-planning lands `lazyspec.rs` structs before the command
  wiring. *Mitigate:* module-level `#![expect(dead_code, reason="…wired in PHASE-NN")]`,
  self-clearing (`mem.pattern.lint.dead-code-self-clearing-leaf`); never bare `allow`.
- **R5 — wire-string mismatch (adversarial F1) — RESOLVED.** Strings verified
  against lazyspec serde (OQ-3): status lowercase / `in-progress`, relations
  `related-to` et al., date `%Y-%m-%d`. The golden encodes them; conformance
  asserts the exact set incl. the date-only form and the unknown-status default.
- **R4 — Synthetic plan id collision.** *Mitigate:* `PLAN-` is unused by any real
  reservation, so `PLAN-NNN` is unique by construction; INV-5 + a test.
- **R6 — lifecycle-vocabulary coupling (SL-028).** The slice status map consumes
  SL-028's FSM vocabulary; SL-028 lands first. *Mitigate:* total map with a `draft`
  default (an out-of-vocab status never breaks the wire); dependency recorded (§3 +
  scope Assumptions); a conformance case feeds an unknown status.
- **R7 — fixture re-triplication (SL-027 / ISS-001).** Re-rolling backlog fixture
  TOML for the golden corpus re-opens the debt ISS-001 just closed. *Mitigate:*
  reuse `write_fixture` via a promoted `pub(crate)` test-support seam (§9); no
  `created = "…"` head literal may reappear in `lazyspec.rs` tests.

## 9. Quality Engineering & Validation

- TDD red/green/refactor.
- **Conformance test** (every-surface, table-driven over the kinds): asserts INV-1..7,
  the keyword renames serialize correctly, the four-string edge vocab, the seven
  status wire strings **including the unknown-status `draft` default**, the
  date-only `%Y-%m-%d` form, stable id-sorted ordering, and that a membered `REQ`
  is absent as a node yet present in its spec body.
- **Corpus construction (SL-027 reuse, CHARGE IX).** The golden corpus is built via
  the real loaders over a temp tree; backlog fixtures **reuse SL-027's
  `write_fixture`/`Fixture`**, promoted to a `pub(crate)` test-support seam (a
  `/consult`-grade visibility promotion — do not improvise at execute). slice / spec
  (+members+reqs) / adr fixtures have **no** unified builder yet; that gap is named
  here and shaped in planning (small per-kind writers beside the promoted backlog
  seam). **No** new `backlog-NNN.toml` head literal is hand-rolled (re-opening
  ISS-001 is forbidden).
- **Golden fixture:** a minimal corpus → expected Brief JSON, value-compared; the
  drift canary. **Deterministic by injection** — the test passes fixed `now`+`version`
  to `project`, so `meta.generated_at`/`doctrine_version` don't make it flaky (the
  purity of `project` is what buys this). The golden encodes the OQ-3-verified wire
  strings.
- **Field-map check vs brief:** every emitted field has a `DocMeta` home.
- **RO proof:** the command is pure read + serialize — no mutation path exists.
- Lint: zero clippy warnings under the repo denials; `just check` before commit.

## 10. Review Notes

### Adversarial self-review (round 1) — integrated

- **F1 → OQ-3, R5.** Status/relation wire *strings* unverified vs lazyspec's serde
  (esp. `InProgress`, `RelatedTo`); a guess passes our suite, breaks at the boundary.
  Now a blocking conformance prerequisite.
- **F2 → §3, §5.1.** "Pure leaf" mislabel corrected: `project` is a pure *function* at
  the command layer (reads command-layer entity structs). ADR-001 holds (forward
  edges, no cycle); purity ≠ leaf-ness.
- **F3 → D3, INV-5, R4, §5.3 table.** Synthetic plan id `SL-NNN~plan` → `PLAN-NNN`,
  grammar-conformant so lazyspec's `{PREFIX}-{NNN}` inference isn't tripped.
- **F4 → OQ-4.** Body for slice/adr/backlog (raw prose vs both-tier synthesis) deferred
  to planning; raw `.md` can be empty and drop structured data.
- **F5 → §5.1.** Reuse forces `spec::render/read_members/read_interactions` to
  `pub(crate)` — a `spec.rs` edit under the behaviour-preservation gate; now stated.
- **F6 → §9.** Golden determinism via injected `now`/`version` made explicit.
- **F7 → OQ-5.** Backlog axis→edge mapping to be enumerated in planning.

Residual unknowns are OQ-3 (blocking, external — needs lazyspec serde) and OQ-4/OQ-5
(planning-time). No governance conflict surfaced; ADR-001/004 alignment confirmed.

### Adversarial self-review (round 2 — inquisition) — integrated

Inquisition `inquisition.md`, against lazyspec source + the adjacent in-flight
slices. Ten charges; all folded in:

- **C-I → §5.2/§5.3, OQ-3, R5.** Wire strings were declared "external/blocking" but
  lie open in lazyspec `document.rs` (cited by line in the brief). Read + verified;
  status table rewritten to **wire strings** (`in-progress`, not `InProgress`);
  OQ-3/R5 resolved.
- **C-II → §5.2, INV-7.** `date` is `%Y-%m-%d` only, not "ISO-8601" — lazyspec's
  `deserialize_naive_date` rejects datetimes.
- **C-III → §5.3, INV-7.** Synthetic plan node `date` sourced from the owning slice's
  `updated` (mandatory, must parse).
- **C-IV → §5.3 edge table, OQ-5.** Backlog axes enumerated (`slices`/`specs`/`drift`
  → all `related-to`); the phantom `blocks` target struck (no axis bears it).
- **C-V → §5.3, §5.4.** `entities[]`/`types[]` sorted before serialization; the
  idempotence claim is now earned.
- **C-VI → §5.3.** The synthetic `plan` TypeDef is hand-authored (no `Kind` const);
  the "built from `Kind` consts" rule's lone exception.
- **C-VII → scope Follow-Ups.** Command renamed `emit-lazyspec-brief` → `export
  lazyspec`; piece-4's `materialize_doctrine_cache` recipe must follow.
- **C-VIII → §5.5.** Dangling outbound targets drop silently (validation
  suppressed) — edge case acknowledged.
- **C-IX → §9, §3, R7.** Golden corpus reuses SL-027's `write_fixture` (promoted to
  test-support); no re-triplication of backlog TOML; slice/spec/adr fixture gap named.
- **C-X → §3, §5.3, R6.** Status map made total (default arm) and rebuilt on SL-028's
  10-state FSM; **SL-028 lands first** — dependency recorded (no typed slice edge
  exists, so prose).

Residual: OQ-4 (body tier, per-kind, planning) and the slice/spec/adr fixture-builder
shape (planning). Lock gate: C-I–V + C-X folded; C-IX resolved before the golden test.
