# IMP-195: Inter-phase golden regression — re-run prior-phase goldens each phase

**Source:** SL-168 postmortem F-2 + §5b.2. **Home:** RFC-005.

Per-phase funnel verify only confirms the *current* phase delta compiles; it never
re-runs prior phases' goldens. A golden authored in an early phase that depends on
corpus/HEAD state drifts silently as later phases commit, surfacing only at audit
(F-2: non-hermetic `e2e_memory_validate_golden` carrying volatile `commits_behind`).

**Fix direction:** after each phase, re-run ALL prior-phase goldens; classify any
break as current-delta vs corpus-drift (flag the latter as non-hermetic).

Related: RFC-005; IMP-196 (hermeticity lint); mem data-only-phase-must-regate.
