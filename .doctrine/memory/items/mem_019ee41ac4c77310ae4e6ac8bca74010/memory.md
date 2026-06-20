# Close-integrate on shared trunk races repeatedly

`/close` stage-2 `dispatch sync --slice N --integrate --trunk refs/heads/main`
is FF-only + expected-tip CAS against the **live** trunk. On a shared `main`
worktree where other agents commit concurrently, two distinct failures bite —
both report-and-halt, never auto-resolve (RSK-010 / SL-125 drive):

1. **Trunk moved mid-command** — `admitted close_target <oid> does not
   fast-forward refs/heads/main (at <newtip>) — trunk moved; create a
   superseding close-target candidate on the new base and re-admit`. The
   admitted candidate's base went stale between `candidate create` and
   `sync --integrate`. Fix: re-`create --supersedes <prior> --base
   refs/heads/main` → re-`admit --review RV-N` → re-`integrate`. To shrink the
   race window, **chain create→admit→integrate in one shell invocation**; the
   candidate ref name is deterministic (`refs/heads/candidate/N/<label>`) so no
   value needs threading. Expect to retry under churn.

2. **`integrate-dirty-worktree (refs/heads/main)`** — integrate FF-resyncs the
   live checkout and refuses a **blanket** dirty tree, even when the dirty file
   is another slice's `.doctrine/` WIP that cannot conflict with the projected
   code units. You may NOT stash/discard another agent's uncommitted work
   (AGENTS.md). Resolution is the work's owner committing it; then re-supersede
   (their commit advanced trunk) and integrate. Net: the close driver cannot
   self-unblock — surface it and wait.

Mandatory `DOCTRINE_TRUNK_REF=main` on every dispatch verb (env doesn't persist
across this harness's shell calls — prefix each); the trunk ladder defaults to
`origin/HEAD`, which lags local `main`.

Related: [[mem.pattern.dispatch.pi-arm-worker-ops]] · siblings
`mem_019ec912f7fd746284bfaef00717443e` (land admitted close_target via
`--integrate --trunk`), `mem_019edd33d3b273928001e6c867cb2de5`
(`--integrate` without `--trunk` is a dry run),
`mem_019ee3d7792b7df19d44b207d3d88a39` (clean exclusive trunk checkout or the
phantom index commits), `mem_019ec473d9f57952954f770b2abcc0ea` (orchestrator on
shared main pays the concurrency cost).
