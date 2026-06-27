# SL-168 pre-design references

## Checks → source map

### 1. Id-integrity
- **Pure fn:** `integrity::id_integrity_findings(root) -> Vec<String>` at `src/integrity.rs:360`
- **Internals:** `check_kind()` (line 211), `scan_kind()` (line 272), `scan_aliases()` (line 320)
- **Kind table:** `integrity::KINDS` (line ~47) — the corpus-wide kind registry
- **CLI wrapper:** `doctrine validate` dispatcher in `src/main.rs` (search `Validate`)
- **Key types:** `KindRef`, `KindSnapshot`, `EntityFacts`, `AliasFacts`

### 2. Spec-FK integrity
- **Pure fn:** `Registry::validate(scope: Option<&str>) -> Vec<String>` at `src/registry.rs:317`
- **Build step:** `build_registry(root)` at `src/spec.rs:1542` (impure — reads filesystem)
- **CLI wrapper:** `run_validate()` at `src/spec.rs:1625`
- **Checks inside:** `dangling_member_fks()`, `dangling_interaction_targets()`, `descent_findings()`, `parent_findings()`, `self_parent()`, `parent_cycle()`, `duplicate_labels()`, `orphan_requirements()`
- **Key types:** `Registry`, `MemberEdge`, `InteractionEdge`, `ParentEdge`, `DescentEdge`, `BuildFinding`

### 3. Memory health
- **CLI entry:** `run_validate()` at `src/memory.rs:3288`
- **Three inline checks:** dangling relations (line ~3307), stale verification (line ~3318), draft expiry (line ~3331)
- **Relation resolution:** `validate_relation_target()` (line ~3357) — tries memory ref, then catalog scan
- **Memory collection:** `collect_all()` for whole-corpus, `resolve_show()` for single
- **Key types:** `Memory`, `MemoryRef`, `Relation`

### 4. Prose citation integrity
- **Primitive:** `scan_danglers(root, needle)` at `src/integrity.rs:581` — finds all cites of ONE ref
- **Token match:** `line_cites()` at line 623 — whole-token `KIND-NNN` detection
- **Disposable skip:** `is_disposable_prose()` at line 609 — excludes `handover.md`, phase notes, etc.
- **Current use:** only called by `run_reseat` to report inbound citations after renumbering
- **Needed inversion:** scan ALL `KIND-NNN` from all `.md` → check each against `ensure_ref_resolves()`
- **Precision concerns:** must exclude backtick-fenced code spans, sentinels (`BOOT-SENTINEL`), and doc-local refs (`D1`, `OQ-1`)

### 5. Done-but-open detector
- **Backlog items:** `src/backlog.rs` — `BacklogItem`, `relation_edges()` (line 959), `targets_for()` for `RelationLabel::Slices`
- **Slice terminal:** `is_transition_terminal()` at `src/lifecycle.rs` (covers `done` + `abandoned`), `is_terminal_status()` at `src/slice.rs:980` (only `done`)
- **Slice status read:** `meta::read_meta()` / `status_and_title_for()` in catalog
- **Design decision needed:** use `is_transition_terminal` (abandoned counts as terminal) or `is_terminal_status` (only done)?

### 6. Raw-label edge detection
- **Relation graph:** `src/relation_graph.rs` — `RelationLabel::Raw(String)` variant
- **Catalog scan:** `outbound_for()` returns edges; raw labels are those that didn't match `RELATION_RULES`
- **IMP-141 context:** 173 edges with `Raw()` labels (156 `related`, 17 `descends_from`)

### 7. Corpus TOML parse integrity
- **Catalog diagnostic channel:** `scan_entities()` at `src/catalog/scan.rs:177` — takes `&mut Vec<CatalogDiagnostic>`
- **CatalogDiagnostic:** `Severity::Error`, `entity_key`, `file`, `field`, `message` — at `src/catalog/scan.rs` (search `struct CatalogDiagnostic`)
- **Per-facet isolation:** `read_facets()` at line 253 — parses `estimate.toml`, `value.toml`, `risk.toml` independently; malformed → `None` + diagnostic
- **Body read:** `read_body()` at line 349 — parses `.md` body, pushes diagnostic on failure
- **Status/title read:** `status_and_title_for()` — hard failure skips entity
- **ScanMode:** `ScanMode { include_bodies: bool }` at line 151
- **Missing:** `plan.toml` under each slice — not read by catalog. `doctrine.toml`, `config.toml` — project-level TOML.
- **IMP-176:** malformed `plan.toml` surfaces only at `slice phases` time, not at authoring

## Design questions carried forward

- **OQ-1:** Should `doctor` accept a `--check` flag to run a specific subset, or always run all?
- **OQ-2:** Should exit code threshold on severity (non-zero only for errors, not warnings) or on any finding?
- **OQ-3:** For done-but-open: use `is_transition_terminal` (abandoned = terminal) or `is_terminal_status` (only done)?
- **OQ-4:** How to register "extra TOML files this kind owns" — new field on `KindRef`? Separate registry?
- **OQ-5:** Should raw-label detection be a sub-check of relation integrity or its own category?

## Related entities

- **IMP-121** — parent backlog item (unified doctor verb)
- **IMP-141** — relation visibility (folded raw-label detection)
- **IMP-176** — plan.toml validation (absorbed into TOML parse integrity)
- **SPEC-004** — entity engine (identity, storage, kind table)
- **SPEC-013** — CLI surface (command grammar, listing spine, canonical id form)
- **SPEC-003** — whole-system context (storage rule, pure/imperative split)

## Relevant memories (retrieved this session)

- `mem.pattern.entity.numbered-kind-identity-table` — KINDS is the one registry; membership is advisory
- `mem.pattern.listing.validate-statuses-is-opt-in` — status validation pattern for list surfaces
- `mem.pattern.lint.cli-handler-args-struct` — args struct pattern for new commands near ceiling
- `mem.pattern.testing.black-box-cli-golden` — pinning CLI output byte-exact
- `mem.pattern.testing.conformance-asserts-surface-not-just-envelope` — test every output surface
