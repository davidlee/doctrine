# Conformance undeclared on a shared branch sweeps in concurrent foreign-slice commits

`doctrine slice conformance <id>` computes its delta over the recorded B..S oid
ranges in `.doctrine/state/slice/<id>/boundaries.toml`. On a **shared branch**
(e.g. the primary `edge` worktree, where multiple slices are authored
concurrently), any commit from *another* slice that lands inside a phase's
`code_start_oid..code_end_oid` range is swept into the `undeclared` union — even
though it is not the slice's work.

**Tell:** the foreign paths cluster under another slice's tree
(`.doctrine/slice/<other>/…`) and the giveaway is in `boundaries.toml` — a phase's
`code_start_oid` is literally a commit *of the other slice*. (SL-170 PHASE-02
`code_start_oid` was the `plan(SL-172)` commit; SL-172's design/toml then showed
up in SL-170's `undeclared`.)

**Audit disposition:** `tolerated`, not a scope-creep finding. There is no per-
slice remediation — the only "fix" is phase isolation (dispatch/worktree per
phase) which is a process choice outside the audited slice. Verify the cause via
`boundaries.toml` before flagging; don't mistake interleave for an undocumented
touch.

Distinct from [[mem.pattern.audit.fork-land-unbound-source-delta]] — that is
the *undelivered* (fork-landed, unbound) case; this is the *undeclared*
(foreign-commit-swept-in) case. Both are shared-history conformance artifacts.
