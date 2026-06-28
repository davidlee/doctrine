# SL-169: Post-Implementation Review — Systemic Improvements

**Slice:** SL-169 — Wire `--columns` on relation list/census + tags in `--columns` + default tags column for all taggable kinds
**Outcome:** `done` — all 5 objectives delivered; code on main @ e16bcf3c
**Review ledgers:** RV-184 (reconciliation audit, 4 findings)
**Scope:** 18 files, 5 design decisions, 5 phases, ~15 agent round-trips over audit→reconcile→close

---

## 1. What Went Well

- **D3's `default_with_tags` helper was the right abstraction.** One 6-line function
  in `listing.rs`, refactored backlog's inline splice onto it first (behaviour-preservation
  proof via unchanged goldens), then reused across 8 kind dispatch sites. DRY, clean,
  and the goldens for unmodified kinds stayed byte-identical.

- **D2's dispatch topology analysis was correct.** Recognizing that `governance.rs`
  serves adr/policy/standard/rfc through ONE column-site edit saved 3 duplicate
  changes and their attendant golden regens. The audit confirmed: only `standard`
  tripped because only its fixture carried a tagged row.

- **The audit earned its keep.** RV-184 found the false-green handover (F-1), the
  undelivered parse-conformance test (F-2), and the 4 undeclared selectors (F-3).
  Each was fix-now in audit scope; none escaped to reconcile as a governance change.

---

## 2. Systemic Friction Points

### 2.1 Dispatch false-green handover (CRITICAL)

The dispatch concluded claiming "2677 pass, 3 pre-existing DOCTRINE_WORKER failures,
1 SL-168 TOML failure." A clean re-run of the candidate showed **zero**
DOCTRINE_WORKER failures — the 3 attributed-to-environment failures were SL-169's
own `e2e_standard_cli_golden` regressions.

**Root cause chain:**
1. The `default_with_tags` helper spliced `tags` into the default column set for
   adr/policy/standard *conditionally* (only when a kind's fixture carries a tagged row).
2. STD-001 carried a tag; the `e2e_standard_cli_golden` suite asserted the *pre-tags*
   column layout and failed.
3. The worker attributed the failures to `DOCTRINE_WORKER` (an env-var-based test
   filter) rather than recognizing them as slice-caused regressions.
4. The `e2e_adr_cli_golden` golden was correctly regenerated (the ADR fixture was
   also tagged), so adr passed — making the standard failure look spurious rather
   than symmetric.

**The false-green pattern is the systemic risk.** A dispatched slice can be handed
off as "complete and green" while carrying regressions that the worker misclassified.
The downstream auditor has no signal to distrust the handover claim; only a clean
re-run of the full suite catches it.

### 2.2 Golden fragility under shared-renderer dispatch (HIGH)

When multiple kinds share a rendering path (`governance.rs` → adr/policy/standard),
a column change to the shared path affects all three kinds' output — but goldens are
per-kind. Only the golden whose fixture triggers the conditional path visibly fails;
the others pass silently. The conditional gate (`any_tagged → splice tags`) means
the set of kinds that regenerate is *data-dependent*, not code-dependent.

**Consequence:** The worker ran the tests, saw 3 failures all from
`e2e_standard_cli_golden`, and attributed them to env. It did not recognize that
`standard`, `adr`, and `policy` share a renderer — that the standard failure was
a proxy for a whole-kind-class regression (any governance kind with a tagged fixture
would fail). If the golden suite didn't have a tagged standard fixture, the
regression would have shipped undetected.

### 2.3 Split-lineage close — PHASE-01 committed to edge directly (MEDIUM)

The `listing::default_with_tags` helper was committed directly to `edge` (c838bb71)
during PHASE-01, while the full 5-phase implementation lived in the dispatch bundle
(`review/169` → `candidate/169/review-001`). At close, this created a split lineage:
edge carried a divergent partial PHASE-01, the candidate carried the complete,
reviewed, audited bundle. The close required pre-FF-ing main to the candidate tip,
resolving the duplicate helper, then converging edge later.

**Root cause:** The dispatch worker executed PHASE-01 directly on edge (its
worktree's `edge` fork) while phases 02–05 landed in the journaled bundle. The
design and plan treat the slice as a single coherent change that should live in
one lineage — but dispatch's phase-at-a-time worktree model means the first phase's
commits land on the worktree's edge fork, not in the bundle, unless the worker
explicitly routes them there.

### 2.4 Selector under-declaration at audit time (PATTERN)

4 paths were undeclared in `design-target` selectors: `src/commands/cli.rs` (relation
`--columns` dispatch wiring), `src/mcp_server/tools.rs` and `src/reconcile.rs` (Rec
`tags` field fallout), `tests/e2e_adr_cli_golden.rs` (adr golden regen). All 4 were
correct, compile-necessary fallout — not scope creep — but undeclared means the
conformance algebra flagged them at audit, requiring a reconcile detour.

**Root cause:** The designer added the `tags` field to Rec's `ListRow` struct
(`src/rec.rs`) and didn't anticipate that this field addition would require edits in
`src/mcp_server/tools.rs` (MCP tool dispatch pattern-matches on struct fields) and
`src/reconcile.rs` (reconcile logic reads Rec fields). These were compile-necessary
fallout, not design choices — the designer couldn't have predicted them without a
compile cycle.

### 2.5 Undelivered work — parse-conformance matrix (MEDIUM)

`e2e_list_conformance.rs` was declared as a design-target and mandated in the plan
(PHASE-05 VT-1: "e2e_list_conformance matrix includes relation + census"), but the
actual test addition only covered the columns golden and not the parse-conformance
matrix. VT-1 was half-satisfied; the dispatch handover didn't catch this because it
tests what exists, not what's missing.

### 2.6 Diffusion of changes (LOW)

18 files for what was described as "pure read-surface wiring." Four of those files
(`cli.rs`, `mcp_server/tools.rs`, `reconcile.rs`, `e2e_adr_cli_golden.rs`) were not
in the original Code-impact table — they were compile-necessary fallout discovered
after the design was locked. The design's file scope was under-estimated by ~22%.

---

## 3. Systemic Improvements

### S1: Dispatch worker — distinguish own regressions from pre-existing failures

**What:** The dispatch worker's test runner must (a) run the suite on a clean trunk
checkout BEFORE implementing any changes, capturing the baseline pass/fail set, then
(b) diff the post-implementation results against the baseline. Any *new* failure is a
slice regression regardless of which test binary or env var it appears under.

**Where:** The dispatch funnel (`/dispatch-subprocess` or `/dispatch-agent` skill,
and the worker's own test script).

**Specifics:**
```
# Baseline (before any slice edits):
cargo test 2>&1 | tee /tmp/baseline.txt
grep "test result:" /tmp/baseline.txt  # record pass/fail counts per binary

# Post-implementation:
cargo test 2>&1 | tee /tmp/current.txt
diff <(failures_from baseline) <(failures_from current)
# Any NEW failure is a slice regression — do NOT attribute to env/pre-existing.
```

The worker must **never** attribute a failure to "pre-existing environmental" without
a baseline diff. The DOCTRINE_WORKER env var may filter which tests run; the worker
must record whether filtering was active and whether the baseline was filtered
identically.

### S2: Golden regeneration — per-kind guard on shared-renderer changes

**What:** When a change modifies a shared renderer (`governance.rs`, `listing.rs`,
`select_columns`, any `COLUMNS` constant), the worker must regenerate **all** goldens
whose kind routes through that renderer — not just the one that visibly fails. The
conditional gate (`any_tagged → splice tags`) makes this non-obvious: only tagged
fixtures trip, but the semantic change affects all kinds in the class.

**Where:** The dispatch worker's golden-update script. Design-time: the design should
name the full set of golden files that need regeneration when a shared renderer
changes.

**Specifics:** For SL-169, the worker should have regenerated `e2e_adr_cli_golden`,
`e2e_policy_cli_golden`, and `e2e_standard_cli_golden` simultaneously because all
three route through `governance::run_list` with `GOV_COLUMNS`. The design's
Code-impact table should enumerate this: "When `GOV_COLUMNS` gains a column, all
governance-kind goldens (adr/policy/standard/rfc) MUST be regenerated, even if
only the tagged subset visibly fails."

### S3: Dispatch — verify VT completeness before handover (NOT just test-results green)

**What:** The dispatch conclude/handover step should run a VT-completeness check:
for each phase's VT criteria, confirm that the delivered tests *exist and cover the
mandated shape*, not just that whatever tests exist pass. SL-169's PHASE-05 VT-1
mandated the parse-conformance matrix; the worker only checked the columns golden.

**Where:** The dispatch funnel's conclude step. Mechanically: after phases complete,
parse `plan.toml` → extract each VT-1 (test) criterion → check that the mandated
test file exists and that its contents match the criterion's expectations. This is
a structural check, not a semantic one — "does `e2e_list_conformance.rs` contain
`relation` and `census` entries?" not "do the tests pass?"

**Specifics:** A lightweight `doctrine slice verify-vt <id>` command that:
```
1. Reads plan.toml → for each phase, collect VT criteria with expects fields.
2. For VT-1 (test): check that the expected test files exist and grep for
   mandated keywords in the criterion's expects string.
3. Report: "PHASE-05 VT-1: expects 'relation + census' in parse-conformance
   matrix — e2e_list_conformance.rs missing relation entry ✗"
```

### S4: Split-lineage prevention — dispatch PHASE-01 must route into the bundle

**What:** The dispatch worktree model should either (a) accumulate all phase commits
into the `review/<slice>` bundle ref (never directly on edge), or (b) the close
skill should detect when edge carries phase commits that the bundle also contains
and handle convergence automatically. The current split-lineage pattern (PHASE-01
on edge, 02–05 in bundle) creates a close-time topology hazard.

**Where:** The dispatch funnel — `dispatch sync --prepare-review` or the per-phase
commit step. After each phase's worktree fork completes, the resulting commits
should land on a staging ref (e.g. `phases/<slice>`) that is squashed into
`review/<slice>` at prepare-review — never committed directly to the worktree's
edge fork.

**Alternative (lighter):** The `/close` skill §3a (dispatched slice integration)
should detect the split-lineage condition (commits on edge's lineage that touch
the slice's declared source paths but aren't ancestors of the admitted candidate)
and offer a convergence recipe rather than discovering it at land time.

### S5: Selector declaration at design time, not audit time

**What:** The conformance algebra catches undeclared selectors at audit — late.
Design should include a step: "run `doctrine slice conformance <id>` (or a dry-run
equivalent) against the planned file set before locking the design." The design
skill should prompt: "Have you declared every file that will be touched, including
compile-necessary fallout in CLI dispatch and MCP tools?"

**Where:** `/design` SKILL.md — add a selector-completeness check as a design-lock
precondition. Or: a new `doctrine check design-selectors <id>` that diffs the
design's Code-impact table against the committed git diff and warns about
undeclared paths.

**Specifics for the skill:**
```
Before locking design.md:
- Enumerate every file you INTEND to touch.
- For each file, ask: "Does this struct/field change force edits in
  CLI dispatch (src/commands/cli.rs)? MCP tool dispatch (src/mcp_server/tools.rs)?
  Reconcile logic (src/reconcile.rs)? Golden files whose kind shares a renderer?"
- Add those fallout files to the Code-impact table NOW, not at audit.
```

### S6: Dispatch handover — include VT-status summary

**What:** The dispatch conclude/handover message should include a structured VT
summary, not just a test-count claim. For each phase:
```
PHASE-01 VT-1: ✓ (unit tests for default_with_tags pass)
PHASE-02 VT-1: ✓ (adr/policy/standard/rfc/knowledge/revision goldens regenerated)
PHASE-05 VT-1: ✗ (e2e_list_conformance.rs missing relation/census coverage)
```

**Where:** The dispatch conclude step — generated by parsing plan.toml VT criteria
and diffing against delivered test coverage. This makes VT gaps visible at handover
time rather than audit time.

---

## 4. Recommendations by Effort

| ID | Effort | Impact |
|----|--------|--------|
| S1 | Medium (worker script change) | **Highest.** Prevents false-green handovers. Every dispatched slice benefits. |
| S2 | Small (worker golden script + design discipline) | Prevents silent golden regressions on shared-renderer slices |
| S6 | Small (handover message format) | Makes VT gaps visible at handover, not audit |
| S3 | Medium (new `verify-vt` command or script) | Catches undelivered work before close |
| S4 | Medium (dispatch funnel change) | Prevents split-lineage close topology hazards |
| S5 | Small (skill guidance) | Reduces undeclared-selector reconcile detours |

**Do now:** S1 (the false-green handover is the #1 systemic risk surfaced by this slice).
**Do next:** S2, S6 (low effort, high recurrence for multi-kind shared-renderer changes).
**Do later:** S3, S4, S5 (structural improvements that harden the process).
