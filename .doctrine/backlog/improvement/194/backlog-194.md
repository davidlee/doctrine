# IMP-194: Diff-aware funnel verify: zero-new across gates+doctor, not just compile

**Source:** SL-168 postmortem §1/F-1,F-3 + §5b.1; SL-169 PIR S1. **Home:** RFC-005.

The dispatch funnel verify only checks compile/clippy/fmt and re-derives *current*
test status. It has no baseline, so three failure classes pass silently:
- A gate that starts RED (`architecture_layering`) can't detect a NEW violation —
  pre-existing-red reads as a pass (F-1: registry→spec upward edge shipped).
- `doctor` isn't run against the corpus at all (F-3: 827 nested-worktree FPs).
- A new test failure is indistinguishable from pre-existing/env (SL-169 S1: a
  slice golden regression misattributed to `DOCTRINE_WORKER`).

**Fix direction:** snapshot the full check-set (tests + gates + doctor findings)
at base `B`, require ZERO new findings post-phase. Generalises SL-170's S1
(test-failure baseline) to gates + doctor. Design may absorb into SL-170.

Related: RFC-005; SL-170 (test-failure case), IDE-008 (executable phase gates),
IMP-198 (hardened layering gate is the other half).
