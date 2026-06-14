# Review RV-030 — reconciliation of SL-064

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-064 — coordination-branch isolation: dedicated
worktree + integration-sync seam for dispatch. Conformance mode, self-audit.
Eight phases implemented (PHASE-01..08), all functionally complete per
`notes.md`; code integrated onto `main`, audit driven from the parent tree
(root), not a worktree fork. **Closure is HELD pending an external code review**
(user directive) — this ledger gathers the reconciliation evidence; `/close` is
deferred.

### Lines of attack

1. **Governance reconciliation (the standing hold, F-PH01-1).** The ADR-006
   D8/D2a/D2b/D9 amendments and the ADR-011 D3/D5/D6/D7 amendments reached
   *self-comparison only*. The slice's own closure intent demands adversarial
   acceptance before close — this is the code-review hold. Does canon now tell
   the truth about coordination-worktree placement + claude-arm base==placement?

2. **Worker-mode fence completeness (OQ-D / D2b residual).** Every new
   write-verb (`worktree coordinate`, `dispatch sync`, `record-boundary`,
   `verify-worker`) must be Orchestrator-classed under the marker-absence fence.
   That fence is defence-in-depth, NOT a coverage proof (RV-025 B3) — the real
   close is IMP-065 (positive marker). Confirm no verb escapes the class.

3. **Working-tree-free projection invariant.** `dispatch sync` must tree-read the
   ledger from the branch tip, never the working filesystem; stage-2 replay needs
   a 3-way CAS (not zero-oid); external refs created via CAS, a stale ref reported
   + journalled, never clobbered; trunk/`edge` never touched.

4. **Behaviour preservation.** git.rs born-frame `capture` byte-for-byte (VT-6);
   `NORMATIVE_FLAGS` single chokepoint preserved; live `.git/index`
   byte-unchanged through `filter_tree`. The existing suites are the proof.

5. **F-1 padding consistency.** `ledger::dispatch_dir` padded to canonical
   3-digit — funnel writer and sync reader must agree on `.doctrine/dispatch/064/`.

6. **Rollup divergence.** `slice list` derives 2/8 from the runtime phase tree,
   but `notes.md` records all eight phases complete — the runtime sheets
   (02–07) were never flipped. Reconcile before the lifecycle move.

7. **Scope discipline / carried follow-ups.** IMP-065 (positive marker),
   IMP-071 (record-orthogonal wiring), IMP-043 (demoted re-anchor) must be
   captured open, not silently dropped (defer-needs-backlog-before-close).

Out of scope: the `e2e_backlog_list_order_golden` gate-red is foreign WIP
breakage (SL-059 `tags` JSON field vs a stale SL-053 golden) — SL-064 touches
zero backlog code; disregarded, not a finding against this slice.

## Synthesis

SL-064 is a high-quality implementation — disciplined pure/impure split, strong
test coverage (89 suites green, behaviour-preservation gate held), no parallel
implementation (coordinate reuses `run_provision` + extracted
`remove_worktree_dir`; sync reuses `filter_tree`/`commit_tree`/`update_ref_cas`),
and the F-1 padding self-catch shows the notes discipline working. Two
independent review passes (codex GPT-5.5 adversarial + a human/Opus full-file
read) plus this reconciliation confirm the locked design is faithfully
mechanised. Of the eight invariants codex attacked, six held outright (tree-based
ledger read, byte-stable live index, 3-way replay CAS, stale-ref report-not-
clobber, F-1 padding, single `NORMATIVE_FLAGS` chokepoint, journal-before-ref
ordering).

**The one close-gating finding — F-1 (blocker).** Stage-1 prepare-review projects
`review/<slice>` and every `phase/<slice>-NN` onto the **live** trunk tip
(`trunk_commit()`), where design §4.2/§4.3 specify the run's pinned
`trunk_base_B`. The coordination worktree isolates the *working tree*, not the
trunk *ref*, so a foreign commit to `main` between `coordinate` and `sync`
silently reparents the projection onto moved trunk — per-phase diffs stop being
exact, and the design's "integrate refuses non-ff" net (§3 / IMP-043) does not
fire because it only covers movement *after* stage-1. Latent (no e2e moves trunk
mid-run). **Dispositioned fix-now (User-ruled):** project off
`merge-base(dispatch/<slice>, trunk)` — the pinned fork-point, no new ledger
state — keeping `trunk_commit()` only at integrate's trunk push. Remediation is
deferred to a `/handover`-driven pass before `/close`; the blocker stays OPEN,
holding the close-gate, until the fix lands and is verified.

**Standing risk consciously accepted — F-2 (tolerated).** The OQ-D Orchestrator-
verb fence rests on marker-*absence*: an unstamped process can invoke
`coordinate`/`sync`/`record-boundary`. This is the documented D2b residual (design
§2/§6/§7) — the verb-class restriction and impersonation tests (the OQ-D plan
obligations) ARE delivered; the real close is the positive coordination marker,
**IMP-065** (open). Defence-in-depth (R-5 import belt + IMP-052 post-spawn check +
env-worker catch + bwrap-no-push) covers v1; the design never claimed coverage by
absence. codex flagged this as a fresh major; it is not — and it wrongly believed
the impersonation tests were missing (they exist: `e2e_dispatch_sync` VT-4,
`e2e_worktree_coordinate` VT-2).

**Fix-now quality batch (before close).** F-4 commit_journal stage message
param; F-5 ScratchIndex cross-PID debris sweep + doc correction; F-8
phase_chain_tip status-filter / clearer error; F-9 read_path_at unit test; F-10
projection_row intentional-equality doc note; and F-3 the runtime phase-sheet
rollup reconcile (slice list 2/8 → 8/8) as part of the lifecycle move.

**Follow-up.** F-7 → IMP-075 (`with_journaled_projection` extraction). Carried
open from the slice: IMP-065 (positive marker), IMP-071 (record-orthogonal
wiring), IMP-043 (sync-time re-anchor, now the named close for the F-1 net),
IMP-072 (WorktreeCreate fail-closability nicety).

**Out of scope (not a finding).** The `just check` gate is red on
`e2e_backlog_list_order_golden` — foreign WIP breakage (SL-059 added a `tags`
JSON field; the SL-053-era golden is stale). SL-064 touches zero backlog code;
its own suites are green.

**Close readiness:** NOT YET. One open blocker (F-1) + four answered fix-now
findings await the remediation pass. `/close` is correctly refused by the
close-gate until F-1 is fixed and verified.
