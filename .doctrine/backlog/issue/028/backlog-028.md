# ISS-028: worker-marker confinement refuses CLI writes in stamped fork, breaking tests that shell the doctrine CLI

Discovered during SL-111 PHASE-02 (commit `83a12e04`).

## Symptoms

`e2e_adr_cli_golden` and `e2e_relation_migration_storage` failed *inside a
stamped worker fork* with `worker fork (signal: marker): refusing authored
write`. Both tests scaffold entities via the `doctrine` CLI, which the
worker-mode marker blocks.

## Root cause

The worker-marker correctly refuses authored writes (its purpose), but tests
that shell the CLI to set up fixtures are collateral damage — the marker can't
distinguish between a worker agent writing authored content and a test fixture
scaffolding entities.

## Verification

Both tests pass green on the markerless coordination tree (10 + 6 green).
These are **not regressions** — the fork sandbox simply isn't the right place to
run CLI-shelling tests.

## Workaround

Trust the post-import coordination verify, not the fork's test result, for
anything that shells the `doctrine` CLI.
