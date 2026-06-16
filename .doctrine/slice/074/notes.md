# SL-074 Notes

## PHASE-01 (completed 3b4ba45)

- `src/concept_map.rs`: 599 lines — CONCEPT_MAP_KIND, scaffold, run_new/list/show, 12 tests
- `install/templates/concept-map.{toml,md}`: DSL-ready scaffold with `dsl = '''\n'''` multiline
- `src/main.rs`: ConceptMapCommand enum + Command::ConceptMap + write_class + main() dispatch
- `src/relation.rs`: Contextualizes label added to RelationLabel enum + RELATION_RULES; CM added to GovernedBy sources
- `src/relation_graph.rs`: Updated no-overlay set to include Contextualizes
- Full suite (1406 tests) green, clippy clean
- Worker imported via funnel: B=186aa1e, S=6e99411, applied onto HEAD=94c2538, committed as 3b4ba45

### Watch-outs for later phases
- `run_show --edges` and `--nodes` print placeholder text — PHASE-02 fills with parsed tables
- `parse_canonical_ref` from integrity.rs handles `CM-001` resolution
- Relation rules: Contextualizes is Unvalidated (freeform target), GovernedBy includes CM
- DSL field is `'''\n'''` multiline — add/remove/rename in PHASE-03 must preserve the multiline format

## PHASE-02 (completed 84be696)

- `src/concept_map.rs`: +~900 lines — parse_dsl, derive_node_key, check, Levenshtein, run_check, extended run_show
- Pure layer: parse_dsl (line-by-line, diagnostics: MalformedLine/EmptyLabel/DuplicateEdge/SelfEdge/CanonicalNodeCollision), derive_node_key, check (SimilarNodeLabel/RelationDrift/EntityRefLike), local Levenshtein (~25 lines Wagner-Fischer)
- `run_show --edges/--nodes` now renders parsed edge/node tables
- `run_check` wired: exits non-zero on structural errors (MalformedLine/EmptyLabel), zero on warnings-only
- Tests: 40 passing (derive_node_key, Levenshtein, parse_dsl, check, integration)
- Full suite green, clippy clean
- Worker imported via funnel: B=3b4ba45, S=5246a38, applied onto HEAD=870b10d, committed as 84be696

### Watch-outs for PHASE-03
- `parse_dsl` is available for duplicate detection (add) and segment matching (rename)
- `toml_edit` round-trip through `get_dsl`/`set_dsl` helpers
- `ConceptMapDoc::dsl` field contains the raw multiline string
- Add: detect exact duplicate edge using parse_dsl, no-op without --force

## PHASE-03 (completed b1af2fa)

- `src/concept_map.rs`: +564 lines — get_dsl/set_dsl (toml_edit round-trip), run_add, run_remove, run_rename_node, 17 tests
- `src/main.rs`: Added Add/Remove/RenameNode variants to ConceptMapCommand, write_class, main dispatch
- get_dsl/set_dsl: parse/edit/stringify via toml_edit, test: [[relation]] rows survive round-trip byte-identical
- run_add: rejects empty segments, exact-duplicate no-op without --force, --force appends anyway
- run_remove: exact case-sensitive trim match, non-zero exit on no match
- run_rename_node: segment-match (split on >, full trimmed segment compare), case-insensitive default, --case-sensitive, --dry-run prints without writing
- Full suite green, clippy clean
- Worker imported via funnel: B=84be696, S=a50a77b, applied onto HEAD=84be696, committed as b1af2fa

### Watch-outs for PHASE-04
- Export renderers are PURE functions over ParsedConceptMap — testable from string fixtures
- Uses parse_dsl from PHASE-02, no new deps
- DOT: node ids are canonical keys (double-quoted), labels/rels escaped, sorted
- Mermaid: synthetic node ids (n_0, n_1...), labels escaped, sorted
- JSON: serde_json::Value, pretty-printed

## PHASE-04 (de facto completed via SL-073 PHASE-06, committed 2d84698)

- Export renderers (`render_dot`, `render_mermaid`, `render_json_value`, `render_json`, escape fns, `ExportFormat`, `run_export`) were imported into the tree via SL-073 PHASE-06 (2d84698), which needed concept-map export for the map explorer
- PHASE-04 worker ran independently in a fork and produced equivalent code, but the SL-073 integration already landed on main
- Implementation verified present: 76 tests (dot_escape, mermaid_escape, render_dot, render_mermaid, render_json, run_export integration). Full suite green, clippy clean
- Export → Read classification in write_class; main dispatch wired
- Branch-point guard caught the concurrent commit; delta verified as already present

## Dispatch summary

| Phase | Worker commit | Funnel commit | Notes |
|---|---|---|---|
| PHASE-01 | 6e99411 | 3b4ba45 | Clean import, applied over SL-073 commits |
| PHASE-02 | 5246a38 | 84be696 | Clean import |
| PHASE-03 | a50a77b | b1af2fa | Clean import |
| PHASE-04 | 3dc490a | (n/a) | Concurrently imported via SL-073 PHASE-06 (2d84698); worker fork redundant |

Slice status: `audit ⚠` (4/4 complete). The ⚠ indicates rollup divergence — PHASE-04 was completed via SL-073 integration rather than the planned worker funnel. Next: `/audit`.

## Audit (RV-037, completed 823df4f)

10 findings raised across code review and design conformance:
- F-1: CLI unreachable — FALSE ALARM (stale binary, not a clap bug)
- F-2: Status model correction (draft/active/done/abandoned → draft/accepted/superseded)
- F-3: Removed --json shorthand from Show
- F-4: Template path correct (design was wrong)
- F-5: Duplicate-edge message now includes line number
- F-6: Misleading test renamed
- F-7: [[relation]] display deferred to follow-up
- F-8: EntityRefLike regex anchored
- F-9: DSL split on ' > ' fragility documented
- F-10: Slug field in template is correct (standard Meta field)

All resolved. RV-038 re-audit confirmed clean.

## Post-audit verification

End-to-end CLI verification: new → list → add → show → check → export (DOT/Mermaid/JSON) → rename-node → remove — all working.

## Implementation stats

- `src/concept_map.rs`: 2507 lines, 76 tests
- `src/main.rs`: ConceptMapCommand (8 variants), write_class + main dispatch
- `src/relation.rs`: Contextualizes label + CM RELATION_RULES
- `install/templates/concept-map.{toml,md}`: DSL-ready scaffold
- No new crate dependencies
- Pure/impure split per ADR-001
