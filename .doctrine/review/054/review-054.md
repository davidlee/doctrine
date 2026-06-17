# Review RV-054 — reconciliation of SL-086

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-086 PHASE-03 (`doctrine status` dashboard, IMP-093) against design.md §4. Lines of attack:

1. **Design conformance** — does the Status struct, assembly, render, and run pipeline match the design's pure/impure split (ADR-001), data sources table, and output shape?
2. **JSON contract** — does `--json` output match the documented shape?
3. **Edge cases** — empty repo, missing boot.md, git failures, graceful degradation on graph/next failures.
4. **Invariants** — D10 (separate sections), D11 (hard needs only), D12 (git log as impure shell), D13 (content-diff staleness).

Reviewed surface: `src/status.rs` (414 + 358 lines), `src/main.rs` churn, `src/boot.rs`/`src/backlog.rs` visibility exports.

## Synthesis

### Closure story

SL-086 PHASE-03 delivers a clean, well-structured `doctrine status` dashboard. Three findings raised, all resolved:

- **F-1 (minor/design-wrong)**: JSON output used flat `blocked_slices`/`blocked_backlog` keys vs design's nested `blocked.slices`/`blocked.backlog`. The design was internally inconsistent — the Status struct definition used flat fields while the JSON example nested them. Resolved by updating the design JSON example to flat keys, matching the struct and the human output's separate sections (D10).
- **F-2 (nit/fix-now)**: Spurious double newline between Boot and Recent commits sections. Fixed by changing `\n\nRecent commits` to `\nRecent commits`.
- **F-3 (minor/fix-now)**: `SliceCounts.blocked` was computed after `bs.truncate(5)`, capping the blocked count at 5 instead of reporting the real total. Fixed by swapping the order: capture total before truncation.

### Standing risks

- **Graceful degradation depends on `unwrap_or_default()`**. The `next` and graph/blocked queries degrade to empty on any failure. This is correct for the dashboard's role (orientation, not enforcement) but means transient parse errors (e.g., SL-082's corrupted TOML) silently suppress blocked/next data. Acceptable tradeoff for a non-gating command.
- **`git log` is a runtime dependency**. The dashboard gracefully suppresses commits if git is absent, but the `--format` string uses an em dash (`—`) that must match the locale. The `parse_git_log` test verifies parseability.

### Tradeoffs consciously accepted

- **Pure/impure split** (ADR-001) holds clean: `assemble_status`, `render_human`, `render_json` are pure; `run` is the impure shell. The boot staleness computation mixes `boot_check` (content-diff) with `std::fs::metadata` (mtime stat) in `run`, but the result is passed as a plain BootSection — the pure layer never touches the clock.
- **Human output length** is 10-20 lines as designed. Suppressed sections (empty blocked, empty commits) keep the output compact.

### Verification status

- 14 unit tests (VT-1 through parse_git_log) all pass; the test suite covers empty corpus, non-empty corpus, JSON shape, boot staleness variants, blocked rendering, next-item display, and git log parsing.
- VA-1 (blocked items respect hard needs only) is verified: the code delegates to `priority::channels::blocked` which exclusively walks the `dep_overlay` (needs edges), confirmed by the priority module's own tests.
- `just check` passes: 1589 tests, clippy zero warnings, cargo fmt clean.
- Smoke-tested human + JSON output on the working repo.
