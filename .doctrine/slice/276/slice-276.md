# Hermetic shared test fixtures

## Context

RV-411 (the CHR-172 sample test-quality review; `doctrine show RV-411`) found
two related weaknesses that can only be fixed together:

- **F-2** — git test fixtures are not hermetic. They inherit the developer's
  global git config and hooks, and `DOCTRINE_*` env. With `commit.gpgsign`
  injected, 91 of 130 `git::tests` fail. `DOCTRINE_TRUNK_REF` or
  `DOCTRINE_AGENT_ID` set breaks others. Production `git::run_git_env` forwards
  the whole process env, so fixture-side `.env()` cannot reach every spawn.
- **F-17** — fixtures are hand-rolled per module instead of shared: 16
  `init_repo` copies and 30 `fn git` copies in `tests/`, and further copies in
  `git.rs`, `worktree/gc.rs`, `worktree/test_helpers.rs`, `state.rs`,
  `dispatch.rs`, `coverage_verify.rs`, `memory.rs` and `review/tests.rs`. So
  hermeticity has no single place to live.

This slice may be reshaped by the systemic test-strategy conversation that
CHR-172's handover sets up. Do not design it before that conversation.

## Scope & Objectives

- One hermetic scratch-repo fixture, hosted where both the bin unit tests and
  the integration tests can reach it (`src/test_support.rs` is already shared
  that way). It pins dates, isolates global/system git config, pins locale, and
  checks exit status on every git call.
- Repo-level isolation for production git spawns under test, e.g. a
  `.cargo/config.toml` `[env]` block (GIT_CONFIG_GLOBAL=/dev/null,
  GIT_CONFIG_NOSYSTEM=1, LC_ALL=C). Verify that it doesn't leak into
  non-test builds.
- `DOCTRINE_AGENT_ID` and similar env reads become injected inputs in
  functions under test, the way `trunk_ladder` already takes its env value.
- Migrate the duplicate fixtures onto the shared one.

## Non-Goals

- The vacuous-test repairs (CHR-173), goldens (IMP-504), and slow live-corpus
  tests (IMP-503). These build on this slice but are routed separately.
- The sha256 zero-OID defect itself (ISS-501). Its dual-format tests may ride
  the new fixture.
- Design e2e fixture consolidation already tracked in IMP-357, IMP-364 and
  IMP-368. Coordinate with those items, don't duplicate them.

## Summary

Make every test repo hermetic by construction by giving it exactly one
implementation.

## Risks, assumptions, open questions

- A `[env]` block also affects `cargo run`. It may need scoping to the test
  profile, or the fixture may have to own the env instead.
- Migration touches many test files, so there is merge-conflict pressure
  against in-flight slices. Consider migrating per module.
- Open: host the fixture in `test_support.rs` (std-only today), or in a dev
  crate?

## Verification / closure intent

The suite stays green under a hostile environment: injected
`commit.gpgsign=true`, a global `core.hookspath`, and `DOCTRINE_TRUNK_REF` /
`DOCTRINE_AGENT_ID` set. Consider making that a standing check. No hand-rolled
`init_repo` or `fn git` fixture remains outside the shared one.

## Follow-Ups
