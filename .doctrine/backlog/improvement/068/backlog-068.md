# IMP-068: SL-040 RV review.rs cleanups: with_turn pre-parsed FindingState, review_cache.rs split, derived_status/cache_staleness single-pass

Pure-quality cleanups in `src/review.rs` surfaced by RV-026 (post-ship
code-review of SL-040). None are correctness defects — code is correct and clear
at current (single-digit) finding counts; these are design/efficiency hygiene
worth one tracked pass when the area is next touched.

- **F-4 (major) — `with_turn` double scan.** Verb closures receive
  `existing: &[FindingRow]` (raw status/severity strings) then re-parse per call
  via `finding_status_of` (linear scan + `parse_finding_status`); the per-turn
  baton reconciliation re-parses again via `finding_states_of`, and
  `doc_unresolved_blockers` does a third string→enum via `Severity::parse`. Pre-parse
  status+severity into `Vec<FindingState>` once inside `with_turn` and pass that to
  the closure; the per-finding gate becomes an index/lookup, not a re-scan.
- **F-6 (minor) — `review.rs` is 3498 lines** housing pure core, authored
  readers, the verb family + baton/lock/CAS, and the warm-cache subsystem (D9,
  D-C10). The warm-cache is conceptually separate (shares only `state_dir` +
  `LockGuard`). Extract `review_cache.rs`. (Overlaps the IMP-025 area.)
- **F-9 (nit) — dead `dedup` in `cache_staleness`.** The three drift classes
  (`changed`/`removed`/`added`) from `ContentSet::diff` are disjoint by
  construction (each path lands in exactly one class), so `dedup` after `sort` can
  never fire. Drop it and assert disjointness, or comment why it stays.
- **F-10 (minor) — `derived_status` sequential scans.** Two `.any()` passes +
  fallthrough; a single-pass fold is marginally faster. Micro-opt at single-digit
  scale (current early-return form is more readable — low priority).

Source: RV-026 findings F-4/F-6/F-9/F-10 against SL-040.
