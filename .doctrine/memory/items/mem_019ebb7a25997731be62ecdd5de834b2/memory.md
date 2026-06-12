# Dispatch verify shared-target false-green: touch + re-run to confirm a fresh compile

The bubblewrap jail shares one `CARGO_TARGET_DIR` (`~/.cargo/doctrine-target-jail/
debug`) across every git worktree. So when a dispatch worker runs its verify
(`env -u DOCTRINE_WORKER just check`) in a fork, the FIRST run can read a compile
artifact produced by another worktree's build — reporting green WITHOUT actually
recompiling the fork's new code or running its new tests (a false green).

**Defeat it:** after a green run, `touch` the files you edited and RE-RUN the
verify; then grep the test output for your new test names to confirm they actually
executed. Only a confirmed-fresh green counts.

**The false-RED twin (SL-050).** The same shared target also pollutes SIBLINGS.
While one dispatch worker compiles a changed binary into the shared target, a
*different* session's `just check` on committed main can run that half-built /
stale binary against the current source's goldens — a false RED (observed:
`e2e_priority_golden` failing `priority.v1` vs `v2`, and a graph test flaking, both
during a concurrent worker's `priority.v1→v2` compile, neither real on committed
HEAD). Before believing a concurrent-session red, confirm committed source vs
goldens are self-consistent, then do a CLEAN rebuild. Here `touch` is NOT enough:
the bin fingerprint itself is poisoned — `rm -rf
~/.cargo/doctrine-target-jail/debug/.fingerprint/doctrine-*` then rebuild forces an
honest relink. Practical orchestrator rule: clear the doctrine fingerprint before
the combined verify whenever a concurrent batch shared the target.

Adjacent to `mem.pattern.build.jail-target-redirect` (the jail target redirect that
makes `./target/debug/doctrine` stale) — same shared-target root cause, different
symptom. See also `mem.pattern.dispatch.worker-verify-unset-doctrine-worker` (the
`-u DOCTRINE_WORKER` on the same verify command) and
`mem.pattern.dispatch.three-way-import-onto-moved-shared-main`.
