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

### Sharper diagnosis (SL-228 PHASE-03, 2026-07-25 — was ISS-240)

The collateral damage is **not inherent**: it is a root-resolution skew. Each
affected e2e sets up its own temp root and drives the CLI with `-p <temp root>`.
That temp root carries **no marker**. But `src/commands/guard.rs:461` resolves
the project root from **CWD** rather than from the explicit `-p`, so it walks up
out of the temp root, finds the *worktree's*
`.doctrine/state/dispatch/worker` marker, and refuses a write that was never
aimed at a marked tree.

So the marker is being tested against a root the command is not operating on.
Threading the explicit `-p` root into the worker-marker lookup would let these
tests pass inside a fork without weakening confinement at all — the guard would
simply check the marker of the tree it is actually writing to. Worth checking
whether other CWD-rooted lookups in `guard.rs` share the skew.

Observed again at SL-228 PHASE-03: ~8 targets red in the worker shell
(`e2e_link_unlink`, `e2e_adr_cli_golden`, `e2e_dep_seq_verbs`, …) while the same
targets passed inside the `worker_commit` gate's own run.

### Why the workaround is more expensive than it looks

Combined with `just validate` short-circuiting to a no-op inside a worker fork
(`justfile:36-40`, SL-225 #1 / DEC-003), a dispatch worker has **no reliable
local green signal at all**: its own `check gate` run is polluted by this skew,
and the recipe the commit gate runs skips the governance legs. The worker
therefore burns a `worker_commit` round trip to discover an ordinary gate
failure — and each such refusal currently returns the whole transcript
(**ISS-219**). The three compound.

## Verification

Both tests pass green on the markerless coordination tree (10 + 6 green).
These are **not regressions** — the fork sandbox simply isn't the right place to
run CLI-shelling tests.

## Workaround

Trust the post-import coordination verify, not the fork's test result, for
anything that shells the `doctrine` CLI.
