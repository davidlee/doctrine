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

All RV-037 findings except F-1 are resolved in commit d000183.

F-1 was a **false alarm** — the binary at `target/debug/doctrine` was stale from
a pre-SL-074 build. After `cargo build` against the correct source, all
concept-map subcommands work correctly: new, list, show, add, remove,
rename-node, check, export (DOT/Mermaid/JSON). End-to-end verification passed.

The audit is clean. Zero valid findings remain. Slice is ready for close.
