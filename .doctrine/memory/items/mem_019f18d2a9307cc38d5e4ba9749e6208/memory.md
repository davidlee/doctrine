# Confine claude isolation:worktree subagents via PreToolUse(Bash) + nested bwrap

Proven on claude-code 2.1.181 (NixOS, bwrap 0.11.2), RSK-014 probe H1. Full
apparatus + raw logs + verdict: `.doctrine/backlog/risk/014/probe-h1/`.

## The mechanism (Bash wall)
A settings-level `PreToolUse` hook (matcher `Bash`) rewrites the command via
`hookSpecificOutput.permissionDecision:"allow"` + `updatedInput.command`,
opaquely wrapping it in a nested bwrap jail: rw-bind ONLY the subagent's
worktree, `--ro-bind / /` everything else. Flags mirror
`scripts/pi-spawn-confined.sh` (the proven pi arm). The live harness **honours
`updatedInput`** — the wrapped command runs, the original never does.
- Wrap **opaquely** (base64 the original, decode+exec inside the jail) — never
  parse the command to inject flags (parsing reintroduces shell-undecidability).
- **Fail closed:** unresolved worktree / missing bwrap / parse error → `deny`.
- Working wrapper: `probe-h1/pretooluse-wrap.sh`.

## Binding facts (PreToolUse stdin) — how to tell who's calling
- `agent_id` present **iff** the call is from a subagent; absent for the
  orchestrator/interactive session. `agent_type` likewise.
- payload `cwd` == the hook process's own `pwd` == the assigned worktree
  (`<root>/.worktrees/agent-<agent_id>`), **stable across concurrent subagents**.
  → anchor bwrap to `cwd`; identity correlation is unnecessary.
- `session_id` is SHARED with the orchestrator (same process) — useless as a
  discriminator. Rule: jail iff `agent_id` present AND `cwd ⊆ .worktrees/`; else
  pass through (orchestrator needs full-repo access). cf. [[mem_019f01e2f7d27fe1886c12ff80811c0c]] (no per-worktree ENV seam — use cwd instead), [[mem_019ec65ecbc77282bad7e10a5240ad27]].

## Edit/Write wall — native covers ONLY the shared checkout
This harness natively denies subagent `Edit`/`Write` to the shared repo checkout
*before any hook runs* ("This agent is isolated in the worktree … Edit the
worktree copy"). Contradicts the old "native covers Bash only" claim — BUT
coverage is **partial**: Writes to `/tmp`, `$HOME`, etc. are NOT caught natively.
A `PreToolUse(Edit|Write)` pathcheck (`realpath(file_path) ⊆ cwd`, fail-closed)
catches those — its deny is proven to enforce in-situ. So full Edit/Write
containment needs BOTH: native (repo) + the pathcheck hook (entire non-repo fs).
NEW machinery to land = bwrap Bash wrapper + Edit/Write pathcheck.

## Operational gotcha
**Settings hooks load at session start ONLY — no hot-reload.** Editing
`settings.local.json` mid-session does nothing until restart (verified; the
"hot-reload" claim is false for this version). Edit/Write are not Bash-gated, so
a broken Bash wrapper is always recoverable via Edit + restart.

## Scope / limits
Linux + bwrap only (macOS seatbelt seam open — IMP-045). Write-containment only:
reads stay open (exfil), network egress unaddressed. To LAND: graduate the
wrapper from `.harness/probe/` into the doctrine skill hooks under a slice.
Related OS-floor confinement on the pi arm: [[mem_019ec345c2d879f3bab52aa1dad7a401]].
