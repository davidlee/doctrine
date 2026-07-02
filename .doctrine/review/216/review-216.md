# Review RV-216 — plan of SL-180

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Lines of attack (5 axes, driven by the inquisitor's mandate):**

1. **VT mandate strength.** Do VT-1 and VT-2 keywords survive a false-green
grep against the current tree, or would the gate falsely attribute passing
before implementation?
2. **Phase sizing/balance.** PHASE-01 = 1 file, PHASE-02 = 5 files. Is
PHASE-02 oversized, or does the coupling argument hold?
3. **EX-3 selector_paths delegation.** Does `selector_paths` (the existing
union reader) have one call-site shape that can delegate to the new
`slice::selectors(…, None)` without changing staleness behaviour?
4. **EX-5 layering claim.** Is the `worktree → slice` edge already present via
`coordinate.rs`, and does the gate's edge-dedup semantics prevent tangle
growth?
5. **Contiguity audit.** Track down the TOML-contiguity evil — which verbs
break the invariant, and is a backlog item already tracking it?

## Synthesis

**Judgement: the plan stands, with two keyword corrections applied. No design
gap found. Cleared for `/phase-plan PHASE-01`.**

### Confirmed-correct (verified against current tree)

- **EX-3 (selector_paths delegation).** `selector_paths` (`src/slice.rs:1866`)
  reads ALL selectors (union, no intent filter) and has ONE caller
  (`src/review.rs:2556`, review-prime staleness). The new
  `slice::selectors(root, id, None)` would return the identical union; the
  delegation is a thin pass-through with zero behavioural change.
- **EX-5 (layering).** `crate::slice::run_phases` in
  `src/worktree/coordinate.rs:235` produces the edge `("worktree", "slice")`.
  The gate extracts edges at top-level-module granularity with deduplication
  (`extract_edges_deduplicates`, line 1065). Adding `crate::slice::selectors(…)`
  in `worktree/mod.rs` produces the SAME pair — the `BTreeSet` collapses it.
  Confirmed by `dump_real_graph`: `worktree -> slice` is present today.
  Command `tangle_baseline` = 123, unchanged. `just gate` is currently green.
- **EX-4 (quotePath ASCII preservation).** Adding `-c core.quotePath=false` to
  the git diff invocation has zero effect on ASCII paths; the existing
  conformance registry suite stays green. No behaviour change for ASCII inputs.
- **`--slice` optional (design §3 F4, §9).** Per design, the belt only fires
  when dispatch skills pass `--slice`; ad-hoc manual imports skip the scope
  check. This is the intended enforcement boundary, not a hole. Promoting
  `--slice` to required is the §9 open question, deferred.

### Findings resolved (plan-level corrections applied)

- **F-1 (blocker) — VT-1 keyword false-positive.** Keywords `["against", "strict"]`
  matched as substrings in current `src/slice.rs` (comments + unrelated code).
  **Fixed:** changed to `["--against", "--strict"]` — the double-dash CLI flag
  spellings, which do NOT exist in the current tree.
- **F-2 (major) — VT-2 keyword gates on production fn, not test.** Keyword
  `["quotePath"]` would match the production `core.quotePath=false` in
  `actual_from_range` without proving a non-ASCII test exists.
  **Fixed:** changed to `["non_ascii", "quotePath"]` — compound match forces
  the test to exercise a non-ASCII path.
- **F-3 (minor) — PHASE-02 oversized.** 5 files vs PHASE-01's 1 file.
  **Tolerated:** the coupling argument holds — `undeclared_paths` IS the
  belt's pure need; splitting creates an artificial handoff. Documented in
  plan.md Accepted Risks.
- **F-4 (minor) — No criterion names double root resolve.** Design §8 accepts
  it as benign. **Fixed:** documented in plan.md Accepted Risks.
- **F-5 (major) — Contiguity audit.** Root cause confirmed:
  `append_relation_row` always appends at end via `array.push(row)`. ISS-058
  already tracks this; updated with confirmed trunk reproduction + secondary
  victim SL-190.

### Standing risks

- `--slice` remains optional — a human running `worktree import` by hand skips
  the scope check. Within the threat model (dispatch funnel always passes
  `--slice`), this is safe.
- Skill dual-root drift: editing `plugins/doctrine/skills/` relies on
  `just reinstall` to propagate. Accepted install hygiene.
- `--against` accepts a valid-but-wrong range (`HEAD~5..HEAD` sweeps unrelated
  commits). Operator responsibility, not a code defect.

### Penance complete. The accused is shriven. Go forth and phase-plan.
