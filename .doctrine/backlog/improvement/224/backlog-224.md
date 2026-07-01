# IMP-224: Replace manual Claude skills/hooks file-copy with claude plugin install

## Summary

Currently `install_for_claude` and `install_hooks_plugin_for_claude` in
`src/skills.rs` manually copy files into `.claude/skills/`. Claude now has
native plugin management — we should use it instead.

## What changes

Instead of manual file-copy, drive Claude's plugin CLI:

```
claude plugin marketplace add $$REPO
claude plugin install doctrine --scope project
```

Where `$$REPO` is the `[install] repo` value from `doctrine.toml` (default:
`davidlee/doctrine`).

Idempotency check: query state before acting:

```
# Is the marketplace registered?
claude plugin marketplace list | grep -A 4 doctrine
# Is the plugin installed?
claude plugin list | grep -A 4 doctrine
```

Only add marketplace / install if missing. Each execution line gets a
y/N/a prompt (the existing `prompt_step` pattern from `run_forward_steps`).

## Affected code

- `src/skills.rs`: `install_for_claude()` (~L765), `install_hooks_plugin_for_claude()` (~L1024)
  — replace or wrap the manual-copy logic with `claude plugin` CLI calls.
- `src/install.rs`: the forward-step orchestration that calls the above.

## Acceptance

- `doctrine install --agent claude` uses `claude plugin add marketplace` +
  `claude plugin install` instead of copying files into `.claude/skills/`.
- Idempotent (no-op when already installed).
- Honour `[install] repo` config value; default to `davidlee/doctrine`.
- Existing tests adapted; new coverage for the plugin path.
- Each of the two `claude plugin` commands gets a y/N/a prompt.
- If the user says no to a step *and* the thing isn't already installed, print a
  final reminder at the end of the install run:
  ```
  Claude Code requires the doctrine plugin. To install:
    claude plugin add marketplace <repo>
    claude plugin install doctrine --scope project
  ```
  (Only print lines for steps that weren't already present.)
- Non-Claude agents unaffected.
