# ISS-260: ADR golden worker-marker skip checks only the env leg

`tests/e2e_adr_cli_golden.rs` defines its own local `skip_under_worker_marker()`
(line 67) whose body is:

```rust
if std::env::var("DOCTRINE_WORKER").is_ok() { … true } else { false }
```

That is a **parallel implementation** of the shared
`test_support::under_worker_marker()`, and a strictly weaker one. The shared
helper reads BOTH legs — env `DOCTRINE_WORKER` OR the marker file at
`.doctrine/state/dispatch/worker` (`test_support.rs:68`, `WORKER_MARKER_REL`).
The local copy reads only the env leg.

## Why it bites

A dispatch fork is marked on disk by `worktree fork --worker`. The repo's own
worker guidance — and `mem.pattern.dispatch.worker-verify-unset-doctrine-worker`
— tells workers to run the green gate as `env -u DOCTRINE_WORKER just gate`,
because the env var otherwise blocks legitimate tempdir entity minting. Doing
exactly that turns the env leg OFF while the marker leg stays ON. The local skip
does not fire, the child `doctrine adr status` is correctly guard-refused, and
three goldens go red:

- `adr_status_transition_prints_exact_and_preserves_edits`
- `adr_status_no_op_prints_but_writes_nothing`
- `adr_status_on_malformed_toml_refuses_and_leaves_file_untouched`

Failure text: ``worker fork (signal: marker): refusing authored write `adr
status` ``.

So `just gate` is **unreachably green in any marked dispatch fork**, for every
slice, on the subprocess arm. The phase exit condition "end green" cannot be
met, and each worker must independently rediscover and rationalise the three
reds — or, worse, "fix" them. Observed on SL-231 PHASE-01; the worker diagnosed
the marker correctly but reported it as inert pre-existing noise.

Confirmed not a delta regression: the same 12 tests pass in the unmarked primary
tree at the same base.

## Fix

Delegate to the shared helper — the local wrapper may stay for its per-test
`eprintln!`, but its predicate must be `under_worker_marker()`:

```rust
fn skip_under_worker_marker(test_name: &str) -> bool {
    if crate::test_support::under_worker_marker() { … true } else { false }
}
```

Then sweep for other local copies of the same predicate. `under_worker_marker`
already carries the doc comment explaining the two legs and why the commit gate
clears the marker (SL-199 F2, SL-225 #2, DEC-003) — the duplicate has drifted
away from that reasoning.

Out of SL-231's touch-set (`tests/e2e_adr_cli_golden.rs` is not one of its
design-target selectors), so it must not ride that slice's delta.

Violates STD-001's single-source principle in spirit and the project's
"no parallel implementation" rule directly.
