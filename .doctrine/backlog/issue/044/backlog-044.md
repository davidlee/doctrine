# ISS-044: dispatch-subprocess spawn breaks: worktree fork status line on stdout pollutes $fork_env

Found driving the SL-133 PHASE-03 dispatch (pi subprocess arm).

`doctrine worktree fork --worker` prints a human status line on **stdout** before
the `KEY=VALUE` env contract, e.g.:

```
provisioned /workspace/doctrine/.dispatch/133-p03: 1 copied, 0 withheld, 0 skipped
CARGO_TARGET_DIR=/home/david/.cargo/doctrine-target-jail/wt/dispatch/133-PHASE-03
```

The `dispatch-subprocess` SKILL template captures this as
`fork_env="$(doctrine worktree fork …)"` then spawns with unquoted `$fork_env`:

```sh
env -C "$D" DOCTRINE_WORKER=1 $fork_env codex exec "…"
```

Word-splitting makes `env` try to exec `provisioned` → **rc 127, worker never
spawns**. Hit live: the first PHASE-03 spawn died instantly this way; worked only
after extracting the `CARGO_TARGET_DIR=` line by hand.

**Fix options (pick one):**
1. `worktree fork` emits the provisioning summary to **stderr**, leaving stdout as
   the pure `KEY=VALUE` env contract (cleanest — the contract is a machine surface).
2. The skill filters fork stdout to env lines: `grep -E '^[A-Za-z_][A-Za-z0-9_]*='`.

Option 1 preferred — the skill template is the documented spawn recipe for both the
codex and pi arms, so the contract should be clean at the source. See memory
`mem.pattern.dispatch.fork-env-contract-stdout-status-line`.
