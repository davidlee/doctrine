# WorktreeCreate plugin hook fires on 2.1.198; -w bypasses it

**Empirically confirmed this session** that a plugin-declared `WorktreeCreate`
hook registers AND fires under Claude Code **2.1.198** (native build). Extends
the earlier "proven 2.1.181" result [[mem_019efa04e19377c0938e58c059507a61]] and
the teardown dependency [[mem_019f1a5ce1f472219da91d0724bb766b]].

## Proof chain (2.1.198)

1. `claude --debug hooks` log: doctrine `hooks.json` loaded `enabled=true`,
   `Registered 6 hooks from 3 plugins` — no drop.
2. A subagent spawned with `isolation: worktree` landed at
   `/<repo>/.worktrees/agent-<agentId>` (doctrine's location, detached at edge
   tip) — **not** the native fallback `.claude/worktrees/<name>`.
3. The harness only diverges from `.claude/worktrees/` when
   `hasWorktreeCreateHook()` is true and it delegates path creation to the hook.
   Divergent location ⇒ the hook fired and supplied the path.
   ("worker fork: no" from `doctrine worktree status` is EXPECTED — a benign,
   non-dispatch spawn is unmarked by design.)

## Sharp edges

- **Native `-w` / `--worktree` bypasses the hook.** In a git repo, `claude -w`
  uses plain `git worktree add` at `.claude/worktrees/<name>` and does NOT invoke
  `WorktreeCreate` (confirmed: `-w` probe produced exactly that, no hook fire).
  The hook only fires on the VCS-agnostic / Agent-`isolation:worktree` path
  (the one doctrine's dispatch-agent relies on). So `-w` is the WRONG probe for
  hook registration.
- **The plugin-hooks `Set([...])` in the binary is TELEMETRY, not the gate.**
  There is a 14-event `hooks: new Set([PreToolUse,…,TaskCompleted])` sitting
  beside sandbox-capability keys; it is byte-identical 2.1.197→2.1.198 and does
  **NOT** contain `WorktreeCreate`. It feeds a field-enumeration/telemetry helper
  (near `tengu_plugin_settings_premature_read`), NOT hook enforcement. Do not
  read its omission of `WorktreeCreate` as "plugin worktree hooks are blocked" —
  they aren't. The real gate is `hasWorktreeCreateHook()` → plugin-hook
  aggregation, which keeps `WorktreeCreate`.
- The binary fully supports `WorktreeCreate` at core level (66 string refs,
  `executeWorktreeCreateHook`). The enum of recognized events includes
  `WorktreeCreate,WorktreeRemove,CwdChanged,FileChanged,MessageDisplay`.

## If it ever "shows none" again

The original 2.1.197 symptom (hook absent from `/hooks`, didn't fire) did NOT
reproduce on 2.1.198. If seen again, it is NOT the telemetry Set. Check, in
order: (1) plugin actually enabled + marketplace registered
[[mem.system.claude.plugin-load-model]]; (2) restricted-mode gate off
[[mem.concept.claude.trust-layers]]; (3) you're triggering via
`isolation:worktree`, not native `-w`; (4) hooks.json actually declares it
(read the loaded file, not the cache).

Trigger the empirical test by spawning an `isolation: worktree` subagent and
checking whether its worktree path is `.worktrees/agent-<id>` (hook fired) vs
`.claude/worktrees/<name>` (native, hook did not). Clean up with `git worktree
remove --force`.
