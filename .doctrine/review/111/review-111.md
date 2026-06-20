# Review RV-111 — reconciliation of SL-125

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (post-implementation). **Self-audit** (raiser==responder).

**Surface reviewed:** the dispatched candidate `cand-125-review-001`
(`candidate/125/review-001` @ `a2320694`) — the no-ff 3-way merge of `review/125`
onto `refs/heads/main`. Its net diff vs `main` is **exactly** `src/worktree.rs`
(+ `primary_worktree` helper; `run_stamp_subagent` R2 source swap) and
`tests/e2e_worktree_stamp.rs` (VT-1, VT-4), +135/-8. The `prime --seed` listed
unrelated main working-tree churn (backlog/124, slice/126, `src/status.rs`,
`slice-125.toml`) — **other agents' work, out of scope.** Evidence refs:
`dispatch/125` `f5af68dd`, `review/125` `05f5a914`. Code commit `9ce7dc0c`.

**Lines of attack:**
1. **R1 untouched (design §2).** Diff the binding anchor / `cwd_valid` /
   `classify_stamp` / `StampRefusal` / the `(Some,Some)` bind — only the slot name
   (`source`→`_anchor`) and the provision call may differ. Any behavioural change
   to R1 is a finding.
2. **R2 correctness (design §3/§4).** `primary_worktree(cwd)` is git's first
   porcelain `worktree` entry, canonicalized; source≠fork now holds for the
   hook-inside-fork case; error folds into the M3 STAMP FAILED path (EX-2).
3. **Refusal preservation (design §5 / VT-3).** Every token still fires; `--path`
   still binding-only (EX-3); cross-repo still `bad-dir` (VT-4, the codex BLOCKER).
4. **Layering / purity (ADR-001, EX-1).** Helper impure in the shell; `classify_stamp`
   stays pure.
5. **Behaviour-preservation (CLAUDE.md gate).** Full bin suite + e2e suites green,
   `just check` clippy-clean, on the integrated candidate surface (not just the
   coordination tree the funnel already verified).
6. **VH-1 honesty.** The fresh-session probe is out-of-suite; Defect C was live for
   this very drive (worker hand-stamped) — confirm that is recorded, not papered over.

## Synthesis

**Closure story.** SL-125 PHASE-01 conforms to its design. The candidate
`cand-125-review-001` net diff vs `main` is exactly the two intended files
(+135/-8); the `src/worktree.rs` diff is line-for-line what design §4a/§4b
specify — the new `primary_worktree` helper, the comment reframed to R1-anchor vs
R2-source, the `(Some(source)…)`→`(Some(_anchor)…)` bind rename, and the provision
act routed through `primary_worktree(&cwd)` with its error folded into the existing
M3 `STAMP FAILED` no-rollback path. **R1 is behaviourally untouched** (the gather
block, `cwd_valid`, `classify_stamp`, the `StampRefusal` match, and the bad-dir
`else` arm are all outside the diff). EX-1..4 met; VT-1 (Defect-C pin) goes
red→green, VT-2 unit-tests the helper, VT-3 refusals stay green, VT-4 preserves
cross-repo `bad-dir` (the codex BLOCKER). Behaviour-preservation holds on the
integrated surface: 2073 bin tests + 11 e2e pass, clippy zero-warnings, fmt clean.

**Standing risks / consciously accepted tradeoffs.**
- **VH-1 deferred (F-2, tolerated).** End-state acceptance — a worker comes up
  stamped with no hand-stamp — is unverifiable until the fix integrates to `main`
  and the orchestrator binary is rebuilt. Defect C was live for this drive (worker
  hand-stamped). VT-1 proves the writer fix in-suite; VH-1 is the harness-level
  confirmation, by design out-of-suite. Re-run the IMP-046 probe post-integration.
- **`just check` not fully green (F-1, tolerated).** `lint-js` fails on a
  pre-existing missing `@eslint/js` in `web/map/`, unrelated to this `.rs`-only
  slice. The Rust gate is fully green.
- **Source byte-equivalence scoped (F-4, tolerated).** Provisioning from primary ==
  from orchestrator holds only for the current one-file allowlist (design §2/A4,
  FU-1). Conditional future work; not in scope.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §Summary + §Follow-ups, slice-125 Summary (F-3):** replace the
  `(to be completed at close)` placeholders with the as-built summary — provision
  SOURCE derived via `primary_worktree` (git first-porcelain entry), R1 anchor
  unchanged, ISS-011 Defect C resolved; carry FU-1 (orchestrator-addressable
  provisioning if the allowlist ever grows divergent untracked state) into
  §Follow-ups.
- **Surface reviewed (F-2 context):** record in the slice that audit reviewed the
  dispatched candidate `cand-125-review-001`, and that VH-1 is deferred to
  post-integration (Defect C live during the drive; worker hand-stamped).

### Governance/spec (REV)
- **None.** No ADR, spec, requirement, or policy change. R1/R2 split, ADR-006
  (orchestrator-sole-writer; marker-absent fail-closed D2a), and ADR-001 layering
  are all unchanged — the slice makes the existing happy path actually stamp.

### Backlog / harvest
- **FU-1 (F-4):** capture the conditional orchestrator-addressable-provisioning
  follow-up as a backlog idea (do not lose it; it has no home yet).
- **RSK-010:** already updated this drive with the setup-time base-staleness
  manifestation + `DOCTRINE_TRUNK_REF=main` workaround.
