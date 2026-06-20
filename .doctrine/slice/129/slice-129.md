# Corpus-wide entity id→path helper (entity::id_path over KINDS)

## Context

The entity id→path formula `<dir>/<NNN>/<stem>-<NNN>.{toml,md}` — seeded in
SPEC-004's storage-rule realisation — is hand-rolled via `format!` at ~85
production sites (plus ~8 test-only) across the codebase. Each copy is an
independent chance to drift the directory layout, and each new kind must
remember to use the right pattern.

The dispatch infrastructure (`outbound_for`, `dep_seq_for`) already treats kind
as data dispatched over `integrity::KINDS` (mem.pattern.entity.kind-is-data-not-trait).
Path construction never got the same treatment.

IMP-067 captures this improvement at the backlog level, re-scoped (per proposal
0004) from a one-arm fix in `dep_seq_for` to a corpus-wide helper.

## Scope & Objectives

1. **Add `stem: &'static str` to `entity::Kind`** — the file-stem for authored
   entity files (`"slice"`, `"revision"`, `"adr"`, etc.). `stem` belongs on `Kind`
   alongside `dir` and `prefix` — all three are fundamental identity properties.
   Marked `#[serde(skip)]` to preserve `doctrine catalog scan --json` output shape
   (RV-117 F-2).
2. **Add `entity::Ext` enum and `entity::id_path`/`rel_path` helpers** — a single
   data-driven function `id_path(root, kind, id, ext) -> PathBuf` encoding the
   `<dir>/<NNN>/<stem>-<NNN>.ext` formula. A shared `make_file_name` internal
   guards against stem-less kinds (RV-117 F-3).
3. **Derive `KindRef::stem` from `kind.stem`** — remove the duplicate `stem` field
   on `KindRef`, reference `kr.kind.stem` at all KINDS rows.
4. **Derive `GovKind::stem` from `kind.stem`** — remove the duplicate `stem` field
   on `GovKind`, reference `kind.stem` throughout `src/governance.rs`.
5. **Replace all production `format!("<stem>-{name}.toml")` sites** (~85 production,
   ~8 test-only) with `entity::id_path(...)` or `entity::rel_path(...)`.
6. **Behaviour-preservation gate** — full `just check` green before (baseline
   recorded) and unchanged after.

## Non-Goals

- **Test assertion full-path strings** (~53 sites like
  `format!(".doctrine/slice/{id:03}/slice-{id:03}.toml")`).
  These are intentionally concrete for readability; abstracting them behind a
  helper would obscure the expected path in test failure output.
- **`meta.rs` internals** (`read_meta`, `read_id`) — already abstracted behind a
  stem parameter; callers pass a kind-tree root, not a project root, so `id_path`
  would double-join `kind.dir`.
- **Dispatch phase-state files** (`phase-{phase_num:02}.toml`) — different path
  formula, not an authored entity file.
- **Memory UUID paths** — named entities with UUID identity, not the
  `<dir>/<NNN>/<stem>-<NNN>` pattern.
- **Trait-ifying `entity::Kind`** — `Kind` stays a data struct with function
  pointer; this adds a field, not a type system change.
- **`BACKLOG_STEM` constant** — kept as-is; harmless.

## Affected surface

**Core declarations (2 files):**
- `src/entity.rs` — add `stem` field to `Kind`, add `Ext` enum, add helpers
- `src/integrity.rs` — remove `stem` from `KindRef`, derive from `kind.stem`

**Kind declarations (36 sites, 11 files — 30 production + 6 test):
- `src/slice.rs` — SLICE_KIND, DESIGN_KIND, PLAN_KIND, NOTES_KIND
- `src/revision.rs` — REV_KIND
- `src/review.rs` — REVIEW_KIND
- `src/rec.rs` — REC_KIND
- `src/rfc.rs` — RFC_KIND
- `src/backlog.rs` — ISSUE_KIND, IMPROVEMENT_KIND, CHORE_KIND, RISK_KIND, IDEA_KIND
- `src/spec.rs` — PRODUCT_SPEC_KIND, TECH_SPEC_KIND
- `src/requirement.rs` — REQUIREMENT_KIND
- `src/concept_map.rs` — CONCEPT_MAP_KIND
- `src/knowledge.rs` — ASSUMPTION_KIND, DECISION_KIND, QUESTION_KIND, CONSTRAINT_KIND
- `src/adr.rs` — ADR_KIND (via GovKind)
- `src/policy.rs` — POLICY_KIND (via GovKind)
- `src/standard.rs` — STANDARD_KIND (via GovKind)
- `src/entity.rs` — 6 test-only Kind values

**KindRef/GovKind stem removal (3 files):**
- `src/integrity.rs` — ~22 KINDS rows drop `stem:` field
- `src/governance.rs` — GovKind struct drops `stem`, 4 GovKind constructors,
  all `g.stem` → `g.kind.stem`

**Replacement sites (~85 production + ~8 test-only, 17 files):**
- `src/slice.rs` (8), `src/revision.rs` (7), `src/review.rs` (5), `src/rec.rs` (2),
  `src/rfc.rs` (1), `src/relation_graph.rs` (2), `src/reconcile.rs` (1),
  `src/governance.rs` (3 — plus ~7 non-path g.stem refs auto-fixed by compiler), `src/backlog.rs` (12), `src/knowledge.rs` (5),
  `src/requirement.rs` (7), `src/spec.rs` (10), `src/main.rs` (8 — test-only),
  `src/lazyspec.rs` (3), `src/catalog/scan.rs` (2), `src/integrity.rs` (3),
  `src/map_server/markdown.rs` (1)

## Risks & assumptions

- **Sub-kinds** (DESIGN_KIND, PLAN_KIND, NOTES_KIND) don't write authored
  `stem-NNN.toml` files. They get `stem: ""` as a sentinel. The shared
  `make_file_name` internal helper (both `id_path` and `rel_path`) guards
  against accidental calls with `debug_assert!`. Release builds rely on caller
  exclusion plus tests.
- **GovKind wrapper** — `stem` field removed, `kind.stem` replaces all `g.stem`
  references. Metadata serialization keys change from `g.stem` to `g.kind.stem`
  at runtime (same string value).
- **Behaviour-preservation** — every replacement produces the identical path the
  `format!` produced. `id_path`/`rel_path` are pure functions over Kind data.

## Verification

1. Baseline `just check` green recorded pre-change.
2. `git diff --stat` confirms only intended files touched.
3. `just check` green after all changes (zero clippy warnings, all tests pass).
4. No test modifications needed — behaviour-preservation by construction.
