# Review RV-017 — reconciliation of SL-057

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation review of the SL-057 PHASE-05 landed surface — the `coverage`
subcommand group (`show`/`record`/`verify`/`forget`). Reviewed the *pure* PHASE-05
delta (`git show eff3ca4`, fork `sl057-phase05`, parent `0846800`), NOT `e3f28c0`
(whose `main.rs` is 3-way auto-merged with concurrent SL-056 work). Two passes:
the `/code-review` skill (embittered-staff lens) and an external adversarial pass
via the codex MCP (GPT-5.5, read-only).

Lines of attack: the two emergent worker decisions (the `load_config` relocation
and the new `canonical_slice_ref` normalization); the access classifier (D2a);
F-VI date injection; the gate-(a) byte-preservation claim; and whether the four
validity-reject goldens assert the real `ValidError` or a proxy.

## Synthesis

**Verdict: acceptable.** The two flagged emergent decisions hold up. The
`load_config` relocation into `coverage_store` (the lower module) is byte-identical
to the PHASE-04 original and kills the store↔verify cycle — ADR-001 clean.
`canonical_slice_ref` is correct and tested in both `--slice 57` and `--slice
SL-057` forms. Gate (a) suites are byte-unchanged; only the sanctioned `coverage
show` view-golden argv churn (b). The four reject goldens assert the *real*
`ValidError` Debug names (`MatcherRequired` / `GlobEscapesTree` / `BadRegex` /
`AliasCommandConflict`), not `is_err()` proxies. F-VI date injection, the per-verb
access classifier, and the loud backfill/withdrawal reporting are all sound.

But the worker normalized the slice and change axes via `canonical_slice_ref` and
**left the requirement axis raw** — the central finding. Four findings raised, all
terminal:

- **F-1 (major, FIXED).** `record`/`forget` stored `--requirement` verbatim while
  the read view canonicalizes its ref (`coverage_view.rs:180`
  `requirement::canonicalize_fk`). So `record --requirement REQ-1` keyed `"REQ-1"`
  and `show REQ-001` (keys `"REQ-001"`) never saw it — a silent key divergence.
  Fixed: both write seams now route `requirement` through the same
  `canonicalize_fk`. New golden `coverage_record_canonicalizes_requirement_ref`
  pins the canonical stored key + a cross-spelling `forget` hit.
- **F-2 (major, FIXED).** `all_slice_ids` used `entries.flatten()`, silently
  dropping per-entry `read_dir` errors — on the *mutating* `verify --all` path a
  skipped slice means stale `Failed`/`Blocked` cells with no diagnostic. Fixed:
  explicit `Result` iteration, errors propagated with path context.
- **F-3 (minor, DEFERRED → IMP-056).** Status rendered via `Debug` into CLI output
  (`InProgress` out vs `in-progress` in); a fix re-pins PHASE-03 `withdrawal_line`
  goldens, so it was deferred out of the audit's in-slice scope.
- **F-4 (nit, FIXED).** `run_record`/`run_forget` parsed the slice ref twice;
  folded into the F-1 fix via a shared `slice_key(u32)` (single SL-NNN source).

The F-1/F-2/F-4 remediation rode the same `slice_key` refactor; `just check` green
(clippy zero, 12 record goldens incl. the new one), `cargo fmt` clean. No design
deviation — F-1 is conformance to §5.3 ("key fields are canonical id strings"),
not scope change.
