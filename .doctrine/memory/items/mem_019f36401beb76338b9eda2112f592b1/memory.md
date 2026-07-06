# Jail RO .git is the bwrap wrap on confined Bash, not an fs permission — PassThrough commits

**Claim.** The "RO shared `.git`" that walls a confined dispatch worktree is **not
a filesystem permission** — it is the **nested bwrap wrap** the `worktree-jail`
PreToolUse hook applies to a *confined* subagent's Bash (`decide_bash` →
`WrapBash`, `src/worktree/pretooluse.rs` / `jail.rs`). An **unconfined** actor —
one whose Bash is `PassThrough` (unwrapped) — sees every `.git` as **RW** and can
commit anywhere.

**Evidence (SL-206 P1, 2026-07-06).**
- Side-probe: from the **main thread**, `touch` succeeded on the git-dir of the
  primary tree, the coord worktree (`.dispatch/SL-206`), AND a `.worktrees/` worker
  gitdir — all RW. Contrast [[mem_019f361983...]] (`workflow-leaf-cannot-commit`):
  a *confined* leaf's Bash sees the same `.git` as RO (`index.lock: Read-only file
  system`). Same fs, opposite verdict ⇒ the RO is the wrap, not the mount.
- P1b: a `dispatch-orchestrator` subagent flipped to `PassThrough` (via a
  `SubagentStart` `agent_id` allowlist) ran Bash **unwrapped** and **committed**
  (empty commit landed, `98835cc`), and read a coord-tree HEAD a jailed agent
  cannot even `git -C`.

**The gate.** `pretooluse::decide()` keys confinement on `agent_id`: none (main
thread) → `PassThrough`; present (any subagent) → confined/wrapped. An **allowlist
that returns `PassThrough` for a nominated `agent_id`** therefore yields a
**committing, unjailed orchestrator** with full git — the mechanism SL-206 A2
rests on. Fail-safe: absent allowlist ⇒ jailed.

**Consequence.** To let a *subagent* commit like the main thread you do **not**
need `worker_commit` or a `DispatchRecord` — you need it **unwrapped** (PassThrough).
`worker_commit` is the *other* bypass (server-side unconfined commit for a still-
jailed worker); nomination is the direct one.

See `.doctrine/slice/206/unjail-direction.md` §2 / §6, and
[[mem.fact.claude.subagentstart-fires-from-child-session]] (the nomination trigger).
