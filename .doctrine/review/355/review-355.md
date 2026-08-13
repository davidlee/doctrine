# Review RV-355 — design of SL-254

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Adversarial pass over `SL-254`'s design at run `dr-019ff653` rev 41 — the
collapse of dispatch's two worker-spawn arms onto one confined subprocess.

**Pre-reading.** `.doctrine/slice/254/design.md` (the subject),
`.doctrine/slice/254/slice-254.md` (scope), `ADR-011` (the ADR this slice
falsifies), `ADR-006` §D2b, `SPEC-021`, `RFC-025`. The design's §10 names where
its own author thinks the attack surface is; treat that as a starting point, not
a boundary.

**Lines of attack.**

1. **The deletion boundary** (§7.2 `D1`–`D3`, `R6`). Ten surfaces are deleted.
   The claim is that each is the claude arm's substitute for something
   `worktree fork --worker` already does. A live consumer on the surviving path
   for `arm-spawn`, `create-fork`'s Fork arm, `verify-worker` or the `Spawn`-row
   recorder falsifies it.
2. **`OQ-1`'s settlement** (§6). Settled at drafting on a census: the fork
   binding's only readers are `worker_commit` and MCP `dispatch_import`. A third
   reader reachable on the retained CLI-import path reopens it.
3. **What the deleted belts were holding.** `worker_commit` carried six belts;
   `DEC-204` says the two non-topological ones stay where they already are
   because `classify_import` survives. Verify that claim against both call sites
   rather than the record's summary.
4. **`INV-2` and the write floor.** The posture claim is *unchanged* — a worker
   still cannot skip a belt. That is what `DEC-211` leaned on to keep the funnel
   cadence out of the `REV`. If the posture does move, the `REV`'s target set is
   wrong.
5. **The governance landing.** `ADR-011` at eight regions, `ADR-006` §D2b's two
   corrections, `SPEC-021`'s `REQ-288` retire / `REQ-291` rewrite, `SPEC-012`
   responsibility 18. Under- or over-counting either way is a finding.
6. **`R4`, the reap divergence.** One generalised script, two harness profiles,
   one structural difference (pi polls for `agent_end`; `claude -p` exits).
7. **Fail-closed reachability** (`DEC-208`). The claim is that no unconfined
   path survives. Look for one.

**Out of scope — do not raise as findings.** Clone provisioning, worker
self-commit, fetch-from-clone, the branch-point guard's re-homing, capsule work,
`REQ-335`'s mediated-write tier, and solo `/worktree` isolation are all
`DEC-213`'s split into `SL-255`. A finding that the design should have covered
them is a finding about that split — raise it as one, explicitly, or not at all.
