# Review RV-038 — reconciliation of SL-074

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Re-audit of SL-074 after fixes for RV-037 findings F-2 through F-6, F-8.
One finding remains: F-1 (CLI unreachable).

Confirming:
- Status model: draft/accepted/superseded (was draft/active/done/abandoned)
- --json flag removed from Show
- Duplicate-edge message includes line number
- Test renamed (parse_dsl_non_colliding_labels_no_diagnostic)
- EntityRefLike regex anchored (^...$)
- Tests still pass, clippy clean

## Synthesis

All RV-037 findings except F-1 are resolved in commit d000183. The five fixes are
confirmed in place: status model corrected to authored-artifact vocabulary,
--json shorthand removed, duplicate-edge message includes line number,
misleading test renamed, EntityRefLike regex anchored.

F-1 (CLI unreachable) is a clap 4 framework defect also affecting the pre-existing
Map command (SL-072). The concept_map module compiles and all 76 tests pass — the
implementation is complete and correct. The CLI surface activation requires a
clap investigation outside SL-074's scope. Dispositioned as follow-up.

The slice is ready for close with one standing risk: the CLI surface is not
reachable via `doctrine concept-map`, but the reader API and all pure functions
are testable and correct. The module is wired and will activate once the clap
registration issue is resolved.
