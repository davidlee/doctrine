# Dispatch integrate --edge advance leg is not FF-gated

The FF (fast-forward) property of `dispatch sync --integrate` advances is **NOT
uniform across legs**. Do not assume an integrate advance is FF-safe.

**Per leg (`src/dispatch.rs`):**

- `advance_row` (:1786) → `advance_pure_ref` (:1821) for a not-checked-out target:
  **plain `git::update_ref_cas`, no `is_ancestor` check.** The CAS guards only
  concurrency (refuses a *moved* target via `expected_old`), NOT ancestry.
- `advance_checked_out` (:1856) for a checked-out target: FF-gated at advance time
  (`git::is_ancestor(expected_old, planned)`, :1863) → `merge --ff-only`.
- Trunk projection rows FF-gate at **plan** time: `plan_trunk_row` (:1980) /
  `plan_candidate_trunk_row` (:2020) `ensure!(is_ancestor(...))`.
- **Edge** projection rows do **NOT** FF-gate at plan time: `plan_edge_row` (:1990)
  and `plan_candidate_edge_row` (:2036) are explicitly commented *"Not ff-gated"*
  (a standing aggregate of local work).

**Consequence:** the `--edge` integrate leg can advance the edge ref to a tree that
is **not a descendant** of the current edge tip. No FF guard blocks a corpus/content
regression on that leg today — the CAS only checks `current == expected_old`, then
sets `planned` whatever its ancestry.

**Why it matters:** this is the live, un-gated path that makes SL-166's **g3**
corpus-clobber gate load-bearing *now*, not merely future insurance for RFC-006's
non-FF trunk integrate. Surfaced as RV-176 F-2 (blocker) against the SL-166 design,
which had falsely claimed both advance legs were FF-only.

See also [[mem.pattern.dispatch.candidate-build-seam]] (the no-ff 3-way candidate
build),  [[mem.fact.doctrine.close-integrate-required]].
