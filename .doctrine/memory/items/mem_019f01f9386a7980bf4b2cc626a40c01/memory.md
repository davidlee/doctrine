# Reopen eviction degrades, never blocks the status transition

`set_phase_status`'s reopen branch (Completed→non-Completed, SL-154 PHASE-03,
design D8/P2-1) clears `code_start_oid` and evicts the registry row via
`forget_source_delta`. The eviction **degrades with a named warning** —
`warn_capture(...)` — it does **NOT** propagate with `?`.

## Why (deliberate departure from the §5.2 pseudocode)

The SL-154 design §5.2 reopen pseudocode writes `forget_source_delta(...)?`, but
propagating breaks two invariants:

- **D5 — the binding must NEVER block a status transition.** A non-repo / bare
  `cwd` makes `boundaries_path` itself error, so `?` would make a plain reopen
  *fail*. Pinned by `state::tests::binding_degrades_without_blocking_when_git_unavailable`
  and `set_phase_status_clears_completed_on_reopen` (both run on non-git roots) —
  add `?` and they go RED.
- It mirrors the **record tail** in the same fn, which already warns-not-propagates
  for exactly this reason.

## Why it's safe to degrade

- **Self-healing:** a lingering (un-evicted) row is overwritten by the
  re-completion's `record_source_delta` upsert (keyed by phase).
- **Loud if abandoned:** a reopened-but-never-recompleted phase surfaces the stale
  row via `registry_completeness` as an `Extra` gap → conformance refuses a clean
  diff. Never a silent garbage pass.

## How to apply

- Keep registry mutations inside `set_phase_status` (`record_source_delta`,
  `forget_source_delta`) on the **degrading** path: `if let Err(e) = … { warn_capture(…) }`,
  never `?`. The status flip is the contract; the binding is additive.
- When design pseudocode shows `?` on a binding-side registry write, treat it as
  illustrative — the overriding D5 non-blocking mandate wins.

Cousin of the capture-degradation guard in `capture_phase_boundary` (same fn family,
same warn-not-block posture).
