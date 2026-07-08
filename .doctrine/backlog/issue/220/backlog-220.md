# ISS-220: vt9 memory-surface test env-sensitive: fails when CLAUDE_PROJECT_DIR resolves an unmasked doctrine root

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`memory::ambient_surface_tests::vt9_no_discoverable_root_emits_nothing`
(src/memory.rs) asserts "no discoverable root ⇒ emit nothing", but
`discover_surface_root` falls back from the (deliberately bogus) payload `cwd`
to the **process env** `CLAUDE_PROJECT_DIR` (src/memory.rs:9629). The test never
isolates that env var, so its premise silently depends on the runner's
environment.

Observed 2026-07-06 (SL-206 PHASE-11 funnel): the `worker_commit` gate suite ran
with `CLAUDE_PROJECT_DIR` set (hook/subagent env) → root discovered → a memory
surfaced for the fixture path → non-empty output → panic → `commit-gate-red`
**false-red** on a delta that never touched `memory.rs`. Confirmed
delta-independent at base 7dcfed82: fails on an untouched tree whenever the env
var points at a doctrine root that lacks a masking seen-set (e.g. the coord
tree); "passes" at `/workspace/doctrine` only because a stale probe artifact
`.doctrine/state/mem-surface-seen-s9.txt` suppresses the surfacing — masked,
not correct.

Fix directions (either suffices):
- have the test inject/neutralize the env fallback (e.g. run the discovery with
  an injected env lookup rather than `std::env::var_os`, per the pure/imperative
  split), or
- gate the fallback out of test scope with an injected `project_dir` param, as
  the sibling worktree nomination tests already do (`act_nominate` pattern,
  SL-206 PHASE-11).

Session-scope artifact `mem-surface-seen-s9.txt` in the primary tree's
`.doctrine/state/` should also be cleaned up — it masks the failure there.

Related: `mem.pattern.dispatch.worker-commit-stale-path-false-red` (the other
known `worker_commit` false-red trigger; this issue is a second, distinct
trigger — an env-sensitive test rather than a stale validation binary).
