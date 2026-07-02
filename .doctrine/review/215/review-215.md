# Review RV-215 — reconciliation of SL-189

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation conformance audit of SL-189 (pi-arm boundary recording scopes
to the imported code commit, not the funnel span). Both phases were driven via
`/dispatch` (claude arm, `dispatch-worker` subagents, opus).

**Reviewed surface (F-2).** The candidate review-surface `candidate/189/review-001`
(tip `b609feed`, impl-bundle merged onto `refs/heads/main`) — the immutable
evidence refs are `review/189` (impl-bundle `b6195574`), `phase/189-01`
(`86083d2d`), `phase/189-02` (`4963f395`). The audited code equals `main` base +
the two funnel commits; `git.rs` (and all non-target files) are byte-identical to
`main`.

**Lines of attack / invariants held:**
1. **The core invariant** — does the pi-arm change make conformance clean *by
   construction* (INV: `git diff S^..S` == `S`'s own patch)? Held via the SL-186
   regression VT-2 + the live `slice conformance 189` dogfood.
2. **Behaviour-preservation gate** (AGENTS.md) — legacy `--start/--end` escape
   hatch + conformance suite green *unchanged* after the `Option<String>`
   migration.
3. **Scope discipline** — pi-only; the already-tight claude arm
   (`run_record_boundary`, `dispatch-agent` prose, router claude bullet) and the
   solo path (`capture_phase_boundary`) untouched.
4. **Clap contract soundness** — the XOR-with-jointly-required-range arg group,
   no silent default.
5. **Deferred gaps stay deferred** — A2/R4/F-3 multi-commit phase and F-2
   batch-scoped rows are consciously out of scope, not silently mis-handled.

## Synthesis

**Outcome: clean. One minor finding (F-1), tolerated; no blockers.**

SL-189 achieves its design goal and is demonstrably correct on the evidence:

- **Core invariant proven three ways.** (a) VT-2 (SL-186 behavioural regression)
  builds `B → refresh-base merge M (foreign src/) → code S (S^==M) → trailing
  .doctrine/ knowledge K` and asserts conformance sees only `S`'s own paths —
  `foreign.rs` (in `S^`) and `K`'s `.doctrine/` paths (after `S`) both excluded.
  (b) The live `slice conformance 189` dogfood: **4 conformant, 0 undelivered, 1
  undeclared** — versus the **17 undeclared** on SL-186 that motivated the slice.
  (c) VT-1 unit proves the `single_commit_boundary` `[S^,S]` derivation and its
  reject-merge / reject-root guards.
- **Behaviour-preservation held.** `check gate` on the candidate surface exits 0
  — full clippy (zero warnings) + the whole test suite green, including the 5
  legacy `--start/--end` e2e cases surviving the `Option<String>` migration and
  the conformance suite. The F-6 ancestor/non-merge guard holds trivially for
  `S^..S`.
- **Clap contract sound.** VT-3 confirms the arg diagnostics against the *text*,
  not just exit codes: `neither_mode_is_a_clap_error`,
  `commit_and_start_together_are_a_clap_error`, `start_without_end_is_a_clap_error`
  all pass. No silent default path.
- **Scope held pi-only.** Orchestrator readback (VA-1) confirmed the claude router
  bullet + `record-boundary` prose are untouched and still record `code_end=S`
  before the knowledge trail; the worker delta touched only `src/state.rs`,
  `src/slice.rs`, `tests/e2e_slice_record_delta.rs` (PHASE-01) and the two pi
  SKILL.md files (PHASE-02) — no `.doctrine/`/`.claude/` leak, no
  `run_record_boundary`/`capture_phase_boundary` edit.

**Standing risks / consciously-accepted tradeoffs (carried from design, not
regressions):**
- **A2/R4/F-3** — a phase split across *multiple* commits (mid-phase
  re-dispatch/reopen) is under-captured by a single `--commit` (phase-keyed upsert
  keeps only the last). Pre-existing (the range path is equally last-write-wins);
  the `--start/--end` escape hatch covers it; the completeness gate still requires
  *a* row. A commit-list-per-phase primitive is honest future work — deferred, not
  introduced here.
- **F-2** — parallel-batch rows are batch-scoped, not per-phase (all phases in a
  batch share `S`); slice-level conformance unions rows, so unaffected.
- **R3** — the claude-funnel and solo (IMP-175) paths still record their spans the
  old way; the helper was built for reuse so their adoption is a follow-up
  one-liner (design D3). Out of scope here.

**Non-finding noted for the record.** A rust-analyzer diagnostic flagged
`git.rs:3946/4064` referencing `DISPATCH_REF_PREFIX` unqualified. Verified false:
`DISPATCH_REF_PREFIX` is defined and used only in `src/dispatch.rs`; `grep` finds
zero references in `git.rs`; the candidate `git.rs` is byte-identical to `main`;
and `check gate` compiled + tested green. Stale cross-worktree LSP indexing noise,
not a defect.

## Reconciliation Brief

### Per-slice (direct edit)
- None. F-1 (the lone undeclared `tests/e2e_slice_record_delta.rs`) was disposed
  **tolerated** with rationale — no metadata write required.

### Governance/spec (REV)
- None. The slice touched no spec, ADR, policy, or standard; ADR-012 (dispatch
  integration topology) governs and is honoured (code-only impl-bundle scope);
  POL-002 / STD-001 unaffected (the one new literal, the clap group id, is a named
  `const`).

**Net:** clean audit, empty write surface. `/reconcile` confirms and advances to
`/close`; `/close` runs stage-2 integrate of `candidate/189/review-001` onto
`main` and admits it against RV-215.

## Reconciliation Outcome

Empty write surface — no-op reconcile pass.

### Direct edits applied
- None.

### REVs completed
- None. SL-189 touched no spec / ADR / policy / standard; ADR-012 (dispatch
  integration topology) governs and is honoured by the code-only impl-bundle
  scope.

### Withdrawn / tolerated
- RV-215 F-1: **tolerated** — `tests/e2e_slice_record_delta.rs` undeclared against
  design-target selectors. Verification scaffolding, not deliverable behaviour;
  rationale in the finding disposition. No metadata write.

Reconcile pass complete — handoff to /close.
