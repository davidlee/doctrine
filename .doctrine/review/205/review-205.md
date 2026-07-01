# Review RV-205 — design of SL-182

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Third-party hostile pass on the **PHASE-05 amendment** to `design.md` (commits
`49dd6144` + `36e5b669`): the funnel pivot from the SubagentStop diff-capture
apparatus (D-funnel-path = Path L) to **symmetric live-import** (`worktree import
--from-worktree`). The amendment reverses a decision that was tried on the ledger
three times (RV-200 F-3, RV-201 F-2, RV-202 M2) on the strength of a **single
live probe** (2026-07-01) that observed the worker `isolation:worktree` tree
persists on disk post-return.

**Lines of attack (the accused's own confession, §5.4 / §5.5 / D-funnel-path /
D-import-verb / INV-6 / ASM):**

1. **The footer `worktreePath` (BLOCKER hypothesis).** §5.4 step 1 + D-funnel-path
   assert the orchestrator "reads `worktreePath` from the [Agent] footer," and on
   that basis declare the RV-202 correlator seam **void**. But `docs/claude/hooks.md`
   records `worktreePath`/`worktree_path` in **only two** roles: the `WorktreeCreate`
   hook *returning* a path to Claude (`:809/:2394/:2436`) and the `WorktreeRemove`
   hook *receiving* it as input (`:2444–:2473`). **No** documented `Agent`-tool
   return/footer field. The PHASE-05 probe proved *tree-persistence*, not footer
   contents. Held to the slice's own **D7** discipline ("docs are hypothesis, probe
   harness facts") the pivot substitutes an **undocumented, unprobed** harness fact
   for a ledger-tried finding — and for **parallel fan-out** (§5.3, N trees off one
   arming) the footer, or a correlator, is *load-bearing*: without it the
   orchestrator cannot bind each Agent return to its tree. RV-202 M2 is not void.

2. **INV-6 enforcement boundary (major hypothesis).** INV-6 claims "no
   WorktreeRemove hook — **enforced** by `verify-worker --dir` fail-close." But
   `verify-worker` proves tree **presence** + HEAD==B + marker, not diff
   **integrity**: a hook (or any process) that cleans untracked / resets the tree
   yet leaves it registered at HEAD==B passes the belt while silently dropping the
   worker delta. The claimed enforcement covers absence, not mutation.

3. **`--force` reap ordering (minor hypothesis).** §5.4 step 4 `git worktree remove
   --force` destroys the worker tree — the sole copy of the imported delta. Is the
   reap gated on import success? If step 3's `git apply --index` partially failed,
   `--force` is unrecoverable data loss. The gate is not stated.

4. **Fork-path parity ASM (probe-acquit candidate).** Probe exercised the
   **Passthrough** path; real dispatch is the **Fork** path. Test whether the ASM
   is timid (persistence is a function of WorktreeCreate-hook *presence* — the same
   `create-fork` binary handles both classify outcomes — so it is *stronger* than
   "assumed identical") or genuinely unpinned until VH-1.

## Synthesis

**Verdict: the pivot is SOUND in its core observation and HERETICAL in one
load-bearing seam.** The teardown premise *was* false, and the probe that
disproved it is clean — the tree persists, the capture apparatus was cost against
a non-problem, and its retirement is righteous. But in tearing out the correlator
the amendment reached for a replacement it never proved.

- **F-1 (blocker) — the confiscated correlator.** The amendment's single gravest
  fault: it declared the RV-202 correlator seam **void** on the strength of a
  footer field (`worktreePath`) that `docs/claude` documents nowhere as an
  Agent-tool return. The heresy is not that the footer is *absent* — it may well
  exist in practice — but that the slice's **own D7 law** ("docs are hypothesis,
  probe the harness") was applied to the retired capture path and *waived* for its
  replacement. Worse, for **parallel fan-out** (§5.3, N trees off one arming) the
  correlator is not decorative: without the footer, N indistinguishable
  `.worktrees/agent-*` trees cannot be bound to their Agent returns. The pivot did
  not *dissolve* the correlator problem; it *assumed it away*. Remediation carries
  a genuine user-steer fork — **probe-the-footer** vs **restore-the-correlator** —
  so it is escalated, not improvised (`/consult`). LEFT OPEN; it correctly gates
  the slice close until the targeting mechanism is proven on the live harness.

- **F-2 (major) — an honest enforcement boundary.** INV-6's "enforced, not merely
  documented" overreaches: `verify-worker --dir` catches tree-*absence*, not
  tree-*mutation*. The true enforcer of "no WorktreeRemove hook" is the installer
  assertion (AF-3), today a mere plan-note. Promote it into the invariant.

- **F-3 (minor) — the ungated reap.** `git worktree remove --force` must be
  conditioned on import success, lest a failed `git apply` followed by `--force`
  immolate the sole copy of the worker's delta. State the ordering invariant; pin
  a VT on the import-fails path.

- **F-4 (minor) — the timid assumption, acquitted upward.** The Fork-path ASM is
  *stronger* than the design dares claim: persistence follows from WorktreeCreate-
  hook presence, and `create-fork` is one binary across both classify outcomes.
  Reword to ground the ASM in mechanism; keep VH-1 as confirmation.

**Standing risk consciously carried:** the live-import funnel's targeting
mechanism (F-1) remains unproven until a Phase-05 harness probe captures the real
Agent return payload. Until then the pivot rests on an article of faith, and the
blocker on this ledger is the teeth that will refuse the slice's close if that
faith is never tested.

**Disposition:** F-2/F-3/F-4 are design-artifact truth-fixes, applied in one
coherent reconcile pass *after* the F-1 steer (to avoid churning §5.4 twice).
F-1 awaits the User's fork before that pass and before any finding is verified.

> **HERESIS URITOR; DOCTRINA MANET**
