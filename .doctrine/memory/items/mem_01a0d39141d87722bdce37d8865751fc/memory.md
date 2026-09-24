# A fresh-binary install rewrites the Claude hook ahead of the PATH binary

`doctrine install` (even `-s <one-skill>`) also rewrites the project's
`.claude/settings.json` hooks to the **installing binary's** CLI shape. The
hook command is `${DOCTRINE_BIN:-doctrine} memory surface …`. With
`DOCTRINE_BIN` unset, the hook runs the `doctrine` on PATH.

When the in-tree build (`./target/debug/doctrine`, per
[[mem.pattern.build.jail-binary-for-skill-install]]) is newer than the PATH
binary, the new hook can carry a flag the PATH binary rejects. Seen in SL-262
PHASE-04: the hook gained `--input claude`, PATH 0.44.5 errored with
`unexpected argument '--input'`, and **every PreToolUse hook failed**. Bash,
Read and Edit were all blocked, so the agent could not repair it from inside
the session.

## How to apply

- Before running install from the fresh binary, check that `DOCTRINE_BIN`
  points at it, or that the PATH binary is at least as new.
- If it has already happened, the user must fix it from outside the tool
  loop: `! git restore -- .claude/settings.json`, or update the PATH
  doctrine, or relaunch with `DOCTRINE_BIN` set.
- A settings.json diff after a skill-only install is install output. Commit
  it or revert it deliberately; don't let it ride into an unrelated commit.
