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
- **bwrap is NECESSARY, proven by control (2026-07-01):** disabling the wrapper
  let a worktree subagent's Bash write EVERYWHERE un-jailed (repo-root, shared
  `.git`, `/tmp`, `$HOME`, host mount-ns); re-arming flipped the same vectors to
  `Read-only file system`. Native imposes **zero** Bash containment — asymmetric
  with Edit/Write (below). No hidden native Bash guard exists to lean on; the
  wrapper is the sole cause of Bash confinement, not a preemptor of a native one.

## Binding facts (PreToolUse stdin) — how to tell who's calling
- `agent_id` present **iff** the call is from a subagent; absent for the
  orchestrator/interactive session. `agent_type` likewise.
- payload `cwd` == the hook process's own `pwd` == the assigned worktree
  (`<root>/.worktrees/agent-<agent_id>`), **stable across concurrent subagents**.
  → anchor bwrap to `cwd`; identity correlation is unnecessary.
- `session_id` is SHARED with the orchestrator (same process) — useless as a
  discriminator. **Fail-CLOSED rule:** pass through ⟺ `agent_id` ABSENT
  (orchestrator). `agent_id` present ⇒ `cwd` MUST be a recognised worktree → jail
  to it; else **deny**. Do NOT frame it as "jail when in a worktree, else pass
  through" — that fails OPEN for the `isolation:none` subagent, which carries
  `agent_id` with `cwd` = repo root (PROVEN 2026-07-01: both gates denied it
  `cwd-not-a-worktree`; the unconfined-subagent arm is stopped at the gate).
  Residual: an undocumented spawn mode with NO `agent_id` would inherit
  orchestrator pass-through; both enumerable modes (worktree, none) carry it.
  cf. [[mem_019f01e2f7d27fe1886c12ff80811c0c]] (no per-worktree ENV seam — use cwd instead), [[mem_019ec65ecbc77282bad7e10a5240ad27]].

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
**Hook *registration* loads at session start ONLY — no hot-reload.** Editing the
`hooks` block in `settings.local.json` mid-session does nothing until restart
(verified; the "hot-reload" claim is false for this version). Edit/Write are not
Bash-gated, so a broken Bash wrapper is always recoverable via Edit + restart.
**But the hook *script body* is re-read every call** — a sentinel-gated branch
inside an already-registered script (e.g. `[ -f DISABLE_WRAP ] && exit 0`) toggles
behaviour LIVE, no restart. Ran the necessity disable→re-arm control + the
fail-open discriminator probe in ONE session this way (saved a full re-onboard).

## Scope / limits
Linux + bwrap only (macOS seatbelt seam open — IMP-045). Write-containment only:
reads stay open (exfil), network egress unaddressed. To LAND: graduate the
wrapper from `.harness/probe/` into the doctrine skill hooks under a slice.
Related OS-floor confinement on the pi arm: [[mem_019ec345c2d879f3bab52aa1dad7a401]].
