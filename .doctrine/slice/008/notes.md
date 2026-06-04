# Notes SL-008: Memory retrieval: find/retrieve + scope ranking + staleness

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — durable decisions (commit 5a826c2)

- **Location-probe match model (NEEDS DESIGN FOLD-IN at close-out).** `match_scope`
  treats the query as a working *location* (`paths ∪ globs`, as path subjects) +
  facets (`commands`, `tags`); a memory matches if its scope ADMITS the location
  via any dimension, highest-specificity dim wins. Per-dim admit rules: paths
  component-prefix (3), globs `**`-aware via the `glob` crate (2), commands
  token-prefix (1), tags set-intersection (0). A query PATH probes both scope.paths
  AND scope.globs. This resolves **codex review F1** + the design open-Q on match
  *direction* — design.md currently pins only the specificity *table*, not the
  direction. **Action:** add a D-decision/addendum to design.md when the slice
  closes (the codex review log at design.md bottom should gain an F1-resolved line).
- **`glob` crate added** (`glob::Pattern::matches`, `**`-aware) as a workspace dep —
  user decision over hand-rolling. A malformed stored pattern = non-match, never a
  reader hard-fail (the store is tool-authored; reader degrades).
- **`days_between` / `parse_ymd` live in `src/retrieve.rs`** (not deferred to
  PHASE-02): one pure YYYY-MM-DD primitive shared by `thread_expiry` and PHASE-02
  staleness/recency — no parallel impl. Already `pub(crate)`; PHASE-02 reuses it.
- **Pure layer parked under module-level `#![expect(dead_code, reason=…)]`** — it
  has no shell caller until PHASE-04, and the expectation self-clears (errors) once
  PHASE-04 wires it. Do not delete it early; do not switch to `#[allow]` (denied).
