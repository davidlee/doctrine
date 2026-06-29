# Review RV-193 — reconciliation of SL-177

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Reviewed surface (F-2).** Candidate interaction branch
`candidate/177/review-001` (worktree `cand-177-review-001`, tip `9df7500e`),
which merges the immutable impl bundle `review/177` (`724ddf02`) onto
`refs/heads/main` (`254b189c`). Evidence refs `review/177` and `dispatch/177`
treated as immutable (R2). Code is NOT yet on trunk — integration is `/close`'s
job, post-reconcile.

**Lines of attack.**

1. **Seam correctness** — is `effective_raw_value(kind, &facets)` the single
   source for entity value in priority, consumed at *both* `base_score.value_dim`
   and the SL-176 burndown `raw_value` site? Authored value wins; value-bearing +
   no facet → `DEFAULT_VALUE = 1.0`; any other valueless kind → `None`. No-clamp:
   authored `0.0`/`< 1.0` returned untouched.
2. **Set fidelity** — `kinds::VALUE_BEARING == [SL] + BACKLOG`, a strict subset of
   `is_work_like` parted by REV. `surface.rs` consumes `is_value_bearing`, local
   `WORK_PREFIXES` gone (no parallel impl).
3. **Conformance algebra** — disposition every undeclared/undelivered cell. Is the
   golden/test blast radius design-anticipated (§9.1) and *complete*, not merely
   green (handover flagged worker under-enumeration)?
4. **Regression gate** — full suite green at S vs green B baseline; build + lint
   clean. Distinguish real regressions from environment artifacts.
5. **Governance** — does the value-absent scoring contract change require a
   spec/REV, or is it internal scoring policy (slice defers ratification "if
   needed")?

**Invariants held.** INV-4 (no raw `f.value` scoring reads remain); behaviour-
preservation gate (unrelated suites green unchanged); storage honesty (default at
the scoring seam, never mutating authored TOML; `value.rs` untouched).

## Synthesis

**Closure story.** SL-177 delivers exactly its scoped change: a single
`effective_raw_value(kind, &facets)` seam in the priority tier (graph.rs) —
authored value wins, a value-bearing kind with no facet defaults to
`DEFAULT_VALUE = 1.0`, every other valueless kind is `None`. The seam is consumed
at *both* declared sites: `base_score`'s `value_dim` (PHASE-01) and SL-176's
burndown `raw_value_of` closure (PHASE-02, the change that makes the default reach
burndown — RV-191 F-1). `kinds::VALUE_BEARING` + `is_value_bearing` are the single
source for the set ([SL]+BACKLOG, strict subset of `is_work_like` parted by REV);
`surface.rs`'s local `WORK_PREFIXES` was promoted away with no parallel impl left
behind. `value.rs` stays authored-facet-pure (RV-191 F-4). No-clamp verified:
authored `0.3` and `0.0` pass through untouched; only *absent* defaults.

**Evidence (positive confirmations — recorded here, not as findings).**

- **Seam + INV-4** — read the full `graph.rs` diff: one helper, two call sites,
  no residual raw `f.value` scoring read. Matches design §5.1 and the VT set.
- **Regression gate GREEN at S.** Built + ran `just check` at the candidate tip
  `9df7500e` (review/177 ⊕ main): every test binary passes — 2808 unit + all e2e,
  **0 failed** across the suite; lint clean, build clean. The re-baselined golden
  set is therefore *complete and correct*, not merely locally green (handover's
  open question, answered).
- **VT coverage** — all seven design VTs are present in the mandated test files:
  scoring default/exclusion/no-clamp (graph.rs), set canaries (kinds.rs),
  burndown F-1 regression guard + non-value-bearing-source exclusion (graph.rs),
  actionability-view set-preservation (surface.rs).

**Standing risks / consciously accepted tradeoffs.**

- **R-4 (user-acked):** valueless work items gain a baseline `value_dim`; bounded
  small (absent-estimate anchor as cost ⇒ `value_dim ≈ 1.0/large`). Intended
  ordering change; the golden reorderings (e.g. RSK-001 ahead of ISS-002) are this
  effect, not a defect.
- **No governance impact.** The value-absent→default is internal scoring policy;
  SPEC-001 governs "values flow; policy owns what they mean" and never spec'd a
  value-absent contract. Slice's "ratification rides reconciliation if needed" —
  it isn't needed. No REV.
- **Environment artifacts (NOT regressions):** (1) the 11 `phase-binding capture
  skipped` warnings are the persistent e2e tmp-dir/`not a git repository` artifact,
  identical at B and S; (2) a fresh worktree build fails on `map_server/assets.rs`
  RustEmbed `Assets::get` until the gitignored `web/map/dist/` frontend bundle is
  provisioned — present in the primary tree, absent in fresh worktrees, unrelated
  to SL-177 (which never touches map_server). Provisioned `dist/` into the
  candidate worktree before the gate.

**Out-of-scope flag (NOT an SL-177 artifact).** The primary worktree carries an
uncommitted edit to `AGENTS.md` (dispatch-ritual reflow + a "DO NOT USE git
checkout <ref> --" line) authored by neither this audit nor the SL-177 impl. Not
in the `review/177` bundle, not committed under SL-177. Shared repo, multiple
agents — left untouched; someone should claim or revert it. Surfaced to the user.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §9.1 (F-2):** correct the blast-radius enumeration to match what was
  actually delivered — *remove* `tests/e2e_priority_cross_kind.rs` (asserts
  ordering/relations, not absolute scores; correctly untouched); *add*
  `tests/e2e_inspect_golden.rs`, `src/priority/channels.rs` (test mod), and
  `tests/fixtures/sl071_inspect_sl00{1,3}_golden.json`.
- **(optional, F-1/F-2) design-target selectors:** consider extending them to
  cover the re-baselined test/golden surface so future `slice conformance 177`
  reports the test edits as conformant rather than undeclared. Bookkeeping only —
  the edits are already design-sanctioned (§9.1); no behaviour at stake.

### Governance/spec (REV)
- None. The value-absent→`DEFAULT_VALUE` default is internal scoring policy;
  SPEC-001 does not spec a value-absent contract; no ADR/spec/REV change required.

## Reconciliation Outcome

### Direct edits applied
- **design.md §9.1 (F-2):** corrected the e2e-goldens blast-radius enumeration —
  removed `tests/e2e_priority_cross_kind.rs` (asserts ordering/relations, not
  absolute scores; correctly untouched), added `tests/e2e_inspect_golden.rs` with
  fixtures `tests/fixtures/sl071_inspect_sl00{1,3}_golden.json`, and noted the
  in-tree `src/priority/channels.rs` test mod re-baseline. §9.1 now matches what
  was delivered.

### REVs completed
- None. No governance/spec change required (brief: value-absent default is internal
  scoring policy; SPEC-001 spec'd no value-absent contract).

### Withdrawn / tolerated
- None. Both findings `verified`; F-1 needed no write (the 5 undeclared edits are
  design-sanctioned), F-2 resolved by the direct edit above.

### Optional item — not done
- Selector-widening (cover the re-baselined test/golden surface so
  `slice conformance 177` reports those edits conformant) deferred by user
  decision. Pure bookkeeping, no behaviour at stake.

Reconcile pass complete — handoff to /close.
