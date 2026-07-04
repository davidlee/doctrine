# SL-197 Design — Add concept (CPT) as a knowledge record kind

Descends from IMP-244 part 1. Adds a seventh `RecordKind` — `Concept` (prefix
`CPT`, tree `.doctrine/knowledge/concept`) — a neutral, addressable home for
local terminology, distinctions, and conceptual anchors. Prose-first: no
epistemic lifecycle, no queryable facet.

## 1. Context — the record-kind surface is guarded scatter, not centralised

`knowledge.rs` §2 and IMP-244 frame "add a record kind" as centralised and
mechanical. It is not centralised (`mem.pattern.doctrine.record-kind-touch-sites`),
but since that memory (4-kind era) the surface has been made **drift-safe**: the
`RecordKind` enum makes the ~10 `knowledge.rs` match arms compiler-forced, and
canaries were added (`combined_constants_cover_record`, the partition ALL-loop,
integrity count pin, `sources_match_shipped_accessors`). A missing CPT now trips a
red test or a compile error at nearly every site.

`kinds::RECORD` is the single source; appending `CPT` auto-cascades to the four
combined constants (`SEARCH_DEFAULT`, `ALL_KINDS`, `TAGGABLE`,
`ADMISSIBLE_DEP_TARGETS`), the relation rules with `sources: RECORD` /
`target: Kinds(RECORD)` (Supersedes, Shapes-source, Spawns-source), and the
`catalog/scan.rs` `outbound_for` dispatch (data-driven via `from_prefix`).

**Residual unsafe scatter — the two composite-union relation sites with no
canary:**
- `relation.rs:530` — Shapes **target** list (`[…many…] ++ RECORD`, hand-spelled)
- `relation.rs:556` — `governed_by` **sources** (`[SL,PRD,SPEC,CM] ++ RECORD ++ BACKLOG`)

Plus two **double-entry** pins that re-spell the record tail as string literals
(`relation.rs:1774` concerns/References, `:1782` Supersedes) — loud on drift, but
two edits per kind instead of one.

## 2. Scope decision (D0) — scoped DRY, two phases

Fold the gap-closing DRY into this slice (not a bare additive CPT), but **exclude**
the `RelationRule.sources` table-shape refactor: the composite unions cannot become
a bare `RECORD` read (Rust stable won't const-concat `&'static` slices), and
restructuring the rule table drags the relation engine's behaviour-preservation
gate for 2–3 sites — low value once canaries exist.

**P1 (derive partition record rows from `knowledge::` vocab) was investigated and
DROPPED as unsound.** Partition's `terminal` is a *distinct* set from knowledge's
`terminal`/`hidden`: EVD partition-terminal = `[confirmed, retracted, superseded]`
but `EVIDENCE_TERMINAL = [retracted, superseded]` (confirmed evidence is
dependency-settled yet still supersedable and visible). Three intentional
partitions of one vocab — collapsing them is a bug, not a cleanup. The `:380`
ALL-loop canary checks only coverage (`gating ∪ terminal == statuses`), correctly
not equality. So partition keeps a hand-written CPT row, guarded by that canary.

### PHASE-01 — behaviour-preserving DRY (6 kinds, no new kind; suites green unchanged)
- **P2** — single-entry the two double-entry pins (`:1774`, `:1782`): re-spelled
  literal tails → `[fixed…] ++ RECORD`. A new kind then edits only the rule + `RECORD`.
- **P3** — add drift canaries for the two unguarded unions: assert the record
  subset of Shapes-target (`:530`) and `governed_by`-sources (`:556`) equals `RECORD`.

The behaviour-preservation gate is the proof: the existing relation/partition
suites must stay green **unchanged** across PHASE-01.

### PHASE-02 — add CPT
Append `CPT` to `RECORD`; fill the compiler-forced `knowledge.rs` data; add the
`integrity::KINDS` row, the `partition.rs` row, the three record-set relation
appends, and the seed template. Every record-bearing site is now compiler-forced,
`RECORD`-derived, or canary-guarded — including the P3 canaries, which will now
require the CPT append to Shapes-target and `governed_by`-sources.

## 3. Kind-design decisions

- **D1 — status vocabulary** `CONCEPT_STATUSES = ["draft", "active", "retired"]`.
  Seed `draft` (first element). `CONCEPT_HIDDEN = ["retired"]`,
  `CONCEPT_TERMINAL = ["retired"]`. Neutral, non-epistemic: a term is jotted
  (draft), fleshed and promoted (active), retired when dead. Drafts stay visible.
  Rejected `refined`/`defined` (IMP-244 sketch) — no behavioural line against
  `active`; vocab bloat (STD-002).
- **D2 — empty `ConceptFacet`** (unit-like struct, `#[derive(Default)]`, **no
  fields**) — intentional, a first in the `RecordFacet` enum, not an oversight.
  IMP-244's sections (Definition, Notes, Distinguish from, Examples, Related) are
  **prose in the md body**. A structured `definition` field would duplicate the md
  Definition section (two sources of truth, drift). `RawFacet` gains no fields;
  `format_facet` emits no `[facet]` block for concepts (empty → suppressed).
- **D3 — Shapes/Spawns membership: keep via RECORD-ride.** CPT auto-joins the
  Shapes/Spawns source-sets. These are *permissive* (may-author, not must); a
  rarely-used edge costs nothing, and carving CPT out would need a bespoke const
  and re-scatter the single source. `concerns` is the semantically-primary edge and
  is added explicitly.
- **D4 — supersede: None (no `supersede.rs` edit).** `supersede_policy`'s `_ => None`
  arm already excludes `CPT`; `validate_matrix` never sees it; `LinkPolicy::LifecycleOnly`
  blocks generic `link --supersedes`. CPT rides `RECORD` in the Supersedes *rule*
  (structural), but the command gate makes it non-supersedable with zero edits —
  identical to HYP. A replaced concept: retire the old, author a `related` edge.

## 4. Code impact — design-target touch-set

PHASE-01 (`src/relation.rs` only):
- `:1774`, `:1782` — replace re-spelled record tail with `RECORD`-derived expected set.
- new canaries — Shapes-target (`:530`) and `governed_by`-sources (`:556`) record subset `== RECORD`.

PHASE-02:
- `src/kinds.rs` — `pub(crate) const CPT: &str = "CPT";`; append `CPT` to `RECORD`;
  update the `groupings_match_documented_membership` RECORD assertion.
- `src/knowledge.rs` — `Concept` enum variant; `CONCEPT_KIND` const; `CONCEPT_STATUSES`
  / `CONCEPT_HIDDEN` / `CONCEPT_TERMINAL`; `ConceptFacet` (empty) + `RecordFacet::Concept`;
  arms in `kind`, `as_str`, `statuses`, `hidden`, `terminal`, `validate_facet`,
  `render_facet`, `format_facet`, `facet_json`, `render_record_toml_seed`; `ALL`
  `[RecordKind; 6]` → `7`; update stale "six"/"four trees" doc comments; update the
  `resolve_ref` expected-prefix help string.
- `src/integrity.rs` — `CONCEPT_KIND` import; `KindRef` row after `HYPOTHESIS_KIND`;
  bump the numbered-kind count pin; add `CPT` to the prefix-collision list.
- `src/relation.rs` — append `CPT` to concerns-sources (`:422`), `governed_by`-sources
  (`:556`), Shapes-target (`:530`); the P2 pins and P3 canaries then pass with the append.
- `src/priority/partition.rs` — `KindPartition { prefix: kinds::CPT, workable: &[],
  gating: &["draft", "active"], terminal: &["retired"] }`.
- `install/templates/knowledge-concept.toml` — new seed (`record_kind = "concept"`,
  `status = "draft"`, `[facet]` empty, `[evidence]` empty, `[relationships]` empty).

**Not edited:** `src/supersede.rs` (D4) — stays a `scope-relevant` fence, not a
`design-target`. `catalog/scan.rs`, the combined constants — auto via `RECORD`/`from_prefix`.

## 5. Verification alignment

- **PHASE-01 behaviour-preservation (VA):** full relation + partition suites green
  **unchanged** — the refactor proof.
- **P3 canaries (VT):** two new tests, record-subset of Shapes-target /
  `governed_by`-sources `== RECORD`. Red before PHASE-02 append if CPT missing.
- **CPT round-trip (VT):** `knowledge new concept "X"` → `CPT-001`; `show`/`inspect`
  (table+json) render the empty facet as no `[facet]` block; seed status `draft`;
  byte-stable toml round-trip.
- **Vocab canaries (VT):** the partition ALL-loop and combined-constants loop cover
  CPT automatically once in `RECORD`/`ALL`; integrity count pin bumped.
- **Relation edges (VT):** `link CPT-001 concerns SL-YYY` accepted; `supersede`
  rejects CPT as NEW and OLD (D4).
- **Seed anti-drift (VT):** template `status = "draft"` == `default_status(Concept)`.

## 6. Invariants & boundary conditions

- CPT counter independent (`CPT-001` ≠ `ASM-001`); prefix load-bearing in `resolve_ref`.
- Empty facet: `validate_facet` builds `RecordFacet::Concept(ConceptFacet::default())`
  from the kind-blind `RawFacet` reading zero fields; `render_facet` emits `\n[facet]\n`
  with no lines (byte-stable vs the empty template), `format_facet`/`facet_json` emit
  an empty facet object.
- CPT never value-bearing (`VALUE_BEARING` excludes `RECORD`) — no value/priority facet.
- Concept as Shapes/Spawns/Supersedes *target*: admitted structurally via `RECORD`;
  supersede gated off by policy (D4).

## 7. Carried assumptions / open questions

- No open design questions. Decisions D0–D4 locked.
- Assumption: `seed_knowledge` test helper is data-driven (`record_kind.as_str()`),
  so CPT needs no test-helper edit — verify at execute; add a CPT test-seed only if a
  web/map test needs a concept fixture.
