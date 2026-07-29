# IMP-352: Fixtures spawn the binary without current_dir, false-redding every worker fork

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

About ten integration targets fail in **any** marked dispatch worker fork, all
with the same message:

```
worker fork (signal: marker): refusing authored write `slice new` —
workers return a source delta; doctrine-mediated writes funnel through the
orchestrator.
```

Observed in SL-233 PHASE-05: `e2e_backlog_filter_alias`, `e2e_dep_seq_verbs`,
`e2e_dispatch_arm_spawn`, `e2e_dispatch_h1_integration`, `e2e_link_unlink`,
`e2e_priority_cross_kind`, `e2e_relation_migration_storage`,
`e2e_revision_dep_seq`, `e2e_spec_req_add_slug`, `e2e_supersede`.

**Cause.** The fixtures spawn the built binary without binding
`.current_dir()` to the fixture root, so the *fork's* worker marker is picked
up and the guard correctly refuses an authored write. The refusal is right; the
fixture is wrong about where it is running.

**Cost, and why it recurs.** A worker cannot use `doctrine check gate` as a
pass/fail signal at all — which is awkward, because phases routinely carry a
`VA-G` that asks for `check gate` exit 0. Each worker instead falls back to
`cargo test --no-fail-fast` and hand-classifies every failure to prove none is
its own delta; the orchestrator then re-verifies on the coordination tree. That
is a per-phase tax on every dispatch worker, paid again in every slice.

**Fix.** Bind `.current_dir(<fixture root>)` in the offending fixtures — a
one-time sweep. A guard test that fails when a fixture spawns the binary with an
unbound cwd would stop it regressing.

Surfaced by the SL-233 PHASE-05 worker; the tax itself predates that slice and
is noted in its handover packet as a standing caveat.
