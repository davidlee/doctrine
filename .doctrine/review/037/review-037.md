# Review RV-037 — reconciliation of SL-074

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Synthesis

SL-074 delivers a well-structured concept-map entity with a clean pure/impure
split and 76 passing tests. The core architecture (Kind registration, DSL parser,
check diagnostics, export renderers, mutation verbs) follows the design's module
layout and function boundaries correctly. The tests cover the important behaviours
and edge cases. Clippy is clean. The `toml_edit` round-trip preserves `[[relation]]`
rows byte-identical.

Three material design deviations and one blocking defect dominate:

**F-1 (blocker): CLI surface unreachable.** The concept-map subcommand is correctly
wired in the Command enum and dispatch but does not appear in `doctrine --help`
and is rejected at runtime. This same defect affects `map` (pre-existing from
SL-072). The code compiles and all 76 unit tests pass, confirming the implementation
works — it simply cannot be invoked. This must be resolved before close.

**F-2 (major): Status model wrong.** The design prescribes `Draft/Accepted/Superseded`
for authored artifacts. The implementation borrowed the work-item lifecycle
`draft/active/done/abandoned` — precisely the vocabulary the design rejected. The
hide-set, list filtering, and the scaffold default are all on the wrong lifecycle.
Correcting this is a planned change with no knock-on breakage (status is free-text
today, but CONCEPT_MAP_STATUSES and is_hidden are the coupling points).

**F-3 (major): `--json` flag on Show.** The design explicitly says no separate
`--json` flag — use `--format json`. The implementation added `--json` as a
shorthand, creating a redundant control path.

The remaining findings (F-4 through F-10) are minor: template path difference
(design was wrong, implementation follows convention), EntityRefLike regex
specificity, missing line number in duplicate-edge message, misleading test name,
missing [[relation]] deserialization, and scaffold slug field addition. None block
close.

**PHASE-04 import verification.** The export renderers landed via SL-073 PHASE-06
(commit 2d84698) in a concurrent import. The code is correct and matches the
design's export section. The branch-point guard verified the delta was present.
No drift.

**Test coverage assessment.** 76 tests cover: scaffold/substitution, parse_ref,
materialise, list (including hide-set), show (table + JSON), derive_node_key
(all edge cases), Levenshtein (classic examples), parse_dsl (empty, valid,
comments, MalformedLine, too-many-segments, EmptyLabel for all three segments,
duplicate, self-edge, collision), check (clean, EntityRefLike, similar label,
relation drift), run_check integration, get_dsl/set_dsl round-trip with relation
preservation, add (empty, duplicate, force, reject empty segments), remove
(positive, not-found, case-sensitive, preserve comments), rename (case-insensitive,
case-sensitive, both source/target, no-substring-match, dry-run), all escape
functions (plain, special chars, combined), render_dot (two-node, empty, escape),
render_mermaid (two-node, empty, escape), render_json (round-trip, empty,
pretty-print), and export integration (DOT, Mermaid, JSON). Good coverage.

**Standing risks.** The CLI registration defect (F-1) is the sole gate before
close. The status model (F-2) should be corrected before acceptance but is not
strictly a close-gate — the slice can ship with the wrong lifecycle and be fixed
in a follow-up. The `--json` flag (F-3) is cosmetic. The EntityRefLike regex
(F-8) could produce false positives on labels like `ABC-123-456` but EntityRefLike
is informational-only, never an error, so impact is low.

**Recommendation.** Fix F-1 (the CLI registration defect is likely a clap enum
ordering issue — try moving ConceptMap before Catalog, or add `#[command(name =
"concept-map")]`). Then fix F-2 (status model). F-3 is optional. Proceed to
/reconcile after F-1 is resolved.

## Brief

Conformance audit of SL-074 (concept-map entity + CLI) against design `a94ad0a`.
The implementation is in `src/concept_map.rs` (2507 lines), wired via
`src/main.rs` Command::ConceptMap, with relation rules in `src/relation.rs`
and `src/relation_graph.rs`. Scaffold templates at `install/templates/concept-map.{toml,md}`.

Lines of attack:
1. CLI surface reachability — can the user invoke concept-map commands?
2. Design conformance — does the implementation match the design's explicit choices
   (status model, command shapes, pure/impure split, export formats)?
3. Test coverage — 76 tests, do they verify the right behaviours?
4. Edge cases — DSL parsing robustness, escape correctness, round-trip integrity.
5. Integration — relation rules, worker guard, write_class classification.
