As of 2026-09-29, test fixtures (`git.rs` ScratchRepo, `common::git`, `LinkedTrees`, and the per-module `init_repo` copies) pin only author and committer dates. Production `git::run_git_env` forwards the whole process environment. Measured under RV-411 F-2:
- With `commit.gpgsign=true` injected through GIT_CONFIG_COUNT/KEY/VALUE, 91 of 130 `git::tests` fail.
- With `DOCTRINE_TRUNK_REF` set, the `trunk_entity_ids_*` tests fail.
- With `DOCTRINE_AGENT_ID` set, `resolve_holder_falls_back_to_git_config_identity` fails.
- A global `core.hookspath` runs on every fixture commit.
- Several fixtures use `.output().unwrap()` without checking the exit status, so a refused commit passes silently.

The only isolating precedent is `src/dispatch.rs` (~11098): GIT_CONFIG_GLOBAL=/dev/null.

How to apply:
- In a new git fixture, set GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1, and assert the exit status.
- Pass env-derived values into the function under test as inputs, as `trunk_ladder` does.
- A suite that is red only on one host: check host git config first.

Fix is SL-276. See also [[mem.pattern.platform.never-export-git-dir-to-a-test-run]].
