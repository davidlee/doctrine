# Dispatch verify shared-target false-green: touch + re-run to confirm a fresh compile

The bubblewrap jail shares one `CARGO_TARGET_DIR` (`~/.cargo/doctrine-target-jail/
debug`) across every git worktree. So when a dispatch worker runs its verify
(`env -u DOCTRINE_WORKER just check`) in a fork, the FIRST run can read a compile
artifact produced by another worktree's build — reporting green WITHOUT actually
recompiling the fork's new code or running its new tests (a false green).

**Defeat it:** after a green run, `touch` the files you edited and RE-RUN the
verify; then grep the test output for your new test names to confirm they actually
executed. Only a confirmed-fresh green counts.

Adjacent to `mem.pattern.build.jail-target-redirect` (the jail target redirect that
makes `./target/debug/doctrine` stale) — same shared-target root cause, different
symptom. See also `mem.pattern.dispatch.worker-verify-unset-doctrine-worker` (the
`-u DOCTRINE_WORKER` on the same verify command) and
`mem.pattern.dispatch.three-way-import-onto-moved-shared-main`.
