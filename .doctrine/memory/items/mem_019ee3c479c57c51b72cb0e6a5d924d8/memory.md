# Dispatch verify: DOCTRINE_TRUNK_REF=main poisons just test-all

DOCTRINE_TRUNK_REF=main belongs only on trunk-resolving dispatch cmds (setup/sync), never on the verify suite — it poisons every test that inits its own temp repo.

**Symptom.** Prefixing `DOCTRINE_TRUNK_REF=main` onto `just test-all` (or `just gate`) yields a wall of failures — ~141 in the doctrine suite — every one panicking with `DOCTRINE_TRUNK_REF=main does not resolve to a commit`. The env var leaks into each test subprocess; tests that `git init` their own fixture repo have no `main` ref there, so trunk resolution throws.

**Why setup/sync need it but verify must not.** In the jail, `dispatch setup`/`sync` resolve trunk via `origin/HEAD`, which lags local `main` (ISS-036), so they need `DOCTRINE_TRUNK_REF=main` to fork/sync off the right base. The verify suite resolves trunk inside each test's own temp repo — there `main` is meaningless and the override breaks it.

**Rule.** Funnel/audit verify is **plain**: `just lint && just test-all && just build`. Scope the env var to the single dispatch command that needs it, e.g. `DOCTRINE_TRUNK_REF=main doctrine dispatch sync …`. The markerless coordination tree resolves trunk natively, so verify needs no override there.

Related: [[mem_019ee083a8ce7ab0966576a6693b5a58]] (worktree branches off trunk default, not HEAD).
