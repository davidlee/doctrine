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

`kinds::RECORD` is the single source, but the cascade splits two ways:
- **Genuinely auto** — the relation rules keyed on `sources: RECORD` /
  `target: Kinds(RECORD)` (Supersedes, Shapes-source, Spawns-source) and the
  `catalog/scan.rs` `outbound_for` dispatch (`from_prefix`) read `RECORD` live,
  so CPT rides them with zero edits.
- **Canary-forced manual** — the four combined constants (`SEARCH_DEFAULT`,
  `ALL_KINDS`, `TAGGABLE`, `ADMISSIBLE_DEP_TARGETS`) are **hand-spelled arrays**
  (kinds.rs:61-79), NOT derived. Appending CPT to `RECORD` turns
  `combined_constants_cover_record` (kinds.rs:113) **red** until CPT is added to
  each — loud, not silent, but four manual edits.

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
ALL-loop canary checks **trinary** coverage (`workable ∪ gating ∪ terminal ==
statuses`, partition.rs:377 via `vocab(prefix)`), not per-set equality — so CPT's
hand-written row passes iff its union equals `CONCEPT_STATUSES`
(`workable []` ∪ `gating [draft,active]` ∪ `terminal [retired]` == the vocab). ✓
So partition keeps a hand-written CPT row, guarded by that canary.

### PHASE-01 — behaviour-preserving DRY (6 kinds, no new kind; suites green unchanged)
- **P2** — single-entry the two double-entry pins (`:1774`, `:1782`): re-spelled
  literal tails → `[fixed…] ++ RECORD`. A new kind then edits only the rule + `RECORD`.
- **P3** — add drift canaries for the two unguarded unions: assert the record
  subset of Shapes-target (`:530`) and `governed_by`-sources (`:556`) equals `RECORD`.
- **P4** — derive the two **runtime** user-facing record-kind-list messages from the
  vocab, killing an unguarded string-drift class: `dep_seq.rs:84` (needs/after
  rejection) and `knowledge.rs:968` (`resolve_ref` unknown-prefix error) build their
  `assumption/decision/…` list from `RecordKind::ALL` (`as_str`/`prefix`), not a
  literal. A canary pins derived == the current 6-kind string. (Clap **doc-comment**
  help prose is a compile-time literal — not derivable; handled in PHASE-02.)

The behaviour-preservation gate is the proof: the existing relation/partition
suites must stay green **unchanged** across PHASE-01 (P4's derived strings equal the
current literals for the 6 kinds, so no user-visible change lands in PHASE-01).

### PHASE-02 — add CPT
Append `CPT` to `RECORD`; fill the compiler-forced `knowledge.rs` data; add the
`integrity::KINDS` row, the `partition.rs` row, the three record-set relation
appends, and the seed template. Every record-bearing site is now compiler-forced,
`RECORD`-derived, or canary-guarded — including the P3 canaries (which now require
the CPT append to Shapes-target and `governed_by`-sources) and the P4 message canary
(now expects the 7-kind derived string). Hand-add "concept" to the two clap
doc-comment help lines (`cli.rs:461`, `knowledge.rs:1650`) — editorial only, since
clap auto-lists the `ValueEnum` variants.

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
  Definition section (two sources of truth, drift). `RawFacet` gains no fields.
  **Seed emits an empty `[facet]` header** (zero fields): the per-kind
  scaffold-order invariant (`record_scaffold_lays_out_toml_md_symlink_per_kind`,
  knowledge.rs:2162) `.expect`s a `[facet]` block for **every** `RecordKind::ALL`
  and pins F1 order meta→`[facet]`→`[evidence]`→`[relationships]`. On-disk
  uniform; `format_facet` **suppresses** the empty block at *display* time (`show`)
  — different render path, no conflict. (A seed with no `[facet]` block would panic
  that existing test — caught by the external review.)
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

PHASE-01:
- `src/relation.rs` — `:1774`, `:1782` re-spelled record tail → `RECORD`-derived
  expected set (P2); new canaries: Shapes-target (`:530`) and `governed_by`-sources
  (`:556`) record subset `== RECORD` (P3).
- `src/commands/dep_seq.rs` — `:84` message: build the record-kind list from the
  vocab instead of the literal (P4) + a canary pinning derived == current.
- `src/knowledge.rs` — `:968` `resolve_ref` error: same vocab-derivation (P4).

PHASE-02:
- `src/kinds.rs` — `pub(crate) const CPT: &str = "CPT";`; append `CPT` to `RECORD`;
  **manually append `CPT` to the four hand-spelled combined constants**
  (`SEARCH_DEFAULT`, `ALL_KINDS`, `TAGGABLE`, `ADMISSIBLE_DEP_TARGETS`) in the
  record-tail position (after `HYP`) — `combined_constants_cover_record` forces
  this; they are NOT auto-derived. Update the `groupings_match_documented_membership`
  RECORD assertion.
- `src/knowledge.rs` — `Concept` enum variant; `CONCEPT_KIND` const; `CONCEPT_STATUSES`
  / `CONCEPT_HIDDEN` / `CONCEPT_TERMINAL`; `ConceptFacet` (empty) + `RecordFacet::Concept`;
  arms in `kind`, `as_str`, `statuses`, `hidden`, `terminal`, `validate_facet`,
  `render_facet`, `format_facet`, `facet_json`, `render_record_toml_seed`; `ALL`
  `[RecordKind; 6]` → `7`; update stale "six"/"four trees" doc comments; update the
  `resolve_ref` expected-prefix help string.
- `src/integrity.rs` — `CONCEPT_KIND` import; `KindRef` row after `HYPOTHESIS_KIND`;
  bump the numbered-kind count pin; add `CPT` to the prefix-collision list.
- `src/relation.rs` — append `CPT` to concerns-sources (`:422`), `governed_by`-sources
  (`:556`), Shapes-target (`:531`); the P2 pins and P3 canaries then pass with the append.
  **Also re-pin the existing explicit Shapes-target assertion (`relation.rs:2131`)** —
  a sorted 19-kind literal list; CPT sorts between `CON` and `DEC`. (`governed_by`-sources
  has no existing full-list test — only the new P3 canary covers it.)
- `src/priority/partition.rs` — `KindPartition { prefix: kinds::CPT, workable: &[],
  gating: &["draft", "active"], terminal: &["retired"] }`.
- `src/commands/cli.rs` — `:461` clap doc-comment prose: already 4-kind stale
  ("assumption / decision / question / constraint" — missing evd/hyp). Bring to
  the current set incl "concept" (editorial). **Coordinate with the concurrent
  `cli.rs` edit in flight.**
- `install/templates/knowledge-concept.toml` — new seed. `record_kind = "concept"`,
  `status = "draft"`, an **empty `[facet]` header** (header line, zero fields — the
  scaffold-order invariant requires it, D2), `[evidence]` + `[relationships]` empty.
  `knowledge.rs:1650` clap prose hand-add "concept".
- **Test goldens (PHASE-02 re-pins):**
  - `tests/e2e_validate_byte_exact_golden.rs:36` — the byte-exact scanned-kind list
    is `ALL_KINDS`; CPT joins it → re-pin the expected string (CPT after HYP).
  - `tests/e2e_knowledge_cli_golden.rs:280` — the unknown-prefix error golden
    (`expected ASM/DEC/QUE/CON/EVD/HYP`) is the same string P4 derives; gains `/CPT`
    in PHASE-02 → re-pin. (`:197` routing loop is hand-listed — no break; optionally
    extend to seed+route a `CPT-001`.)
  - `record_scaffold_lays_out_toml_md_symlink_per_kind` (knowledge.rs:2162) stays
    green **unedited** — the empty `[facet]` header keeps CPT conformant (D2).
- `install/using-doctrine.md:49` — "six kinds" → "seven"; embedded distribution
  asset (ships in the binary via RustEmbed), so stale text is *shipped* drift
  (editorial).

**Not edited:** `src/supersede.rs` (D4) — stays a `scope-relevant` fence, not a
`design-target`. `catalog/scan.rs`, the combined constants, `search.rs`, `tag.rs`,
`dep_seq::is_record`/`ADMISSIBLE_DEP_TARGETS` — auto via `RECORD`/`from_prefix`.

## 5. Verification alignment

- **PHASE-01 behaviour-preservation (VA):** full relation + partition suites green
  **unchanged** — the refactor proof.
- **P3 canaries (VT):** two new tests, record-subset of Shapes-target /
  `governed_by`-sources `== RECORD`. Red before PHASE-02 append if CPT missing.
- **P4 message canary (VT):** derived record-kind-list string (dep_seq + resolve_ref)
  == the expected vocab join. In PHASE-01 pins the 6-kind string unchanged; in
  PHASE-02 tracks CPT automatically.
- **CPT round-trip (VT):** `knowledge new concept "X"` → `CPT-001`; `show`/`inspect`
  (table+json) render the empty facet as no `[facet]` block; seed status `draft`;
  byte-stable toml round-trip.
- **Vocab canaries (VT):** the partition ALL-loop and combined-constants loop cover
  CPT automatically once in `RECORD`/`ALL`; integrity count pin bumped.
- **Relation edges (VT):** `link CPT-001 concerns SL-YYY` accepted; `supersede`
  rejects CPT as NEW and OLD (D4).
- **Seed anti-drift (VT):** template `status = "draft"` == `default_status(Concept)`;
  scaffold-order loop (knowledge.rs:2162) green **unedited** — CPT seed carries the
  empty `[facet]` header.
- **Golden re-pins (PHASE-02):** `e2e_validate_byte_exact_golden` (scanned list +CPT)
  and `e2e_knowledge_cli_golden:280` (unknown-prefix +/CPT) re-pinned to 7-kind
  strings. Both stay green **unchanged** through PHASE-01 (P4 derives the current
  6-kind string), flip only on the PHASE-02 append.

## 6. Invariants & boundary conditions

- CPT counter independent (`CPT-001` ≠ `ASM-001`); prefix load-bearing in `resolve_ref`.
- Empty facet: `validate_facet` builds `RecordFacet::Concept(ConceptFacet::default())`
  from the kind-blind `RawFacet` reading zero fields. VT-1 round-trip (the test-only
  hand-emit `render_record_toml`): a Concept record renders `\n[facet]\n` with no field
  lines, parses back to `ConceptFacet::default()`, re-renders identically — byte-stable.
  This is independent of the seed template (production seed emits an empty `[facet]`
  header per D2; VT-1 checks render→parse→render, not seed==render).
  `format_facet`/`facet_json`: `show` suppresses
  the empty `[facet]` block; `facet_json` emits `{}` for the concept facet.
- CPT never value-bearing (`VALUE_BEARING` excludes `RECORD`) — no value/priority facet.
- Concept as Shapes/Spawns/Supersedes *target*: admitted structurally via `RECORD`;
  supersede gated off by policy (D4).

## 7. Carried assumptions / open questions / notes

- No open design questions. Decisions D0–D4 + P4 locked.
- **External review (codex/GPT-5.5) integrated.** Caught: (1) the scaffold-order
  invariant (knowledge.rs:2162) requires a `[facet]` block per kind → D2 seed now
  emits an empty `[facet]` header (was: omit); (2) two integration goldens re-pin
  in PHASE-02 (`e2e_validate_byte_exact_golden`, `e2e_knowledge_cli_golden:280`);
  (3) `install/using-doctrine.md:49` ships stale. Self-caught alongside: the four
  combined constants are canary-forced manual adds, not auto-cascade (§1, §4).
  Confirmed sound: D4 supersede gate; the trinary partition canary + CPT row.
- **P3 `governed_by` canary is a new assertion** — it codifies "every record kind is a
  `governed_by` source" (true today). Deliberate and loud: a future record kind that
  should NOT be governance-governed would trip it, forcing an explicit carve-out rather
  than silent divergence.
- **String-drift class:** internal doc-comments already stale at EVD/HYP
  (`supersede.rs:11/21/41`, `relation.rs:418/3189`, `integrity.rs:923`, `dep_seq.rs:31/63`,
  `map_server/markdown.rs:6`) still say `ASM/DEC/QUE/CON`. Out of scope to chase all
  (cosmetic, non-user-facing); P4 fixes only the two **user-facing runtime** messages.
  Fixing the rest belongs to the deferred DRY-the-scatter backlog item.
- Assumption: `seed_knowledge` test helper is data-driven (`record_kind.as_str()`),
  so CPT needs no test-helper edit — verify at execute; add a CPT test-seed only if a
  web/map test needs a concept fixture.
- Web/map (PRD-015) is a non-goal (IMP-244 defers concept-map/web UI). Verify at execute
  that no Rust exhaustive `record_kind` match in `map_server` breaks the build; the scan
  dispatch is data-driven (`from_prefix`), so no break expected.
