# WorktreeCreate payload cwd is an orchestrator-controlled per-spawn channel

**Proven on claude-code 2.1.181 (wtc-cwd probe, 2026-06-25; SL-152 design).**
Two facts about an `Agent isolation:worktree` spawn, decisive for hook design:

## P3 — payload `cwd` follows the orchestrator's Bash cwd

The `WorktreeCreate` payload `cwd` is **the orchestrator session's current
working directory at spawn time**, not the session launch root. In Claude Code's
harness the **Bash-tool cwd persists across tool calls**, and `cd` shifts it — so
the orchestrator can park its cwd anywhere and every subsequent
`isolation:worktree` spawn carries that path as payload `cwd`.

Empirics: spawn from `/workspace/doctrine` → payload `cwd=/workspace/doctrine`;
after `cd .dispatch/SL-123`, next spawn → payload
`cwd=/workspace/doctrine/.dispatch/SL-123`. Payload stays thin:
`{session_id, transcript_path, cwd, hook_event_name, name:"agent-<hex>"}` — no
`agent_type`, base, or target path (consistent with
[[mem_019ec093bd7b71518489dd187b77f0f0]]).

**Consequence:** `cwd` is a deterministic, per-spawn, transient channel the
orchestrator controls. A `WorktreeCreate` hook can discriminate *positionally* —
"is payload `cwd` a dedicated arming dir?" — instead of via a persistent on-disk
marker. Each coord tree is its own git worktree, so
`git -C <payload.cwd> --show-toplevel` resolves the coord-tree root even from a
subdir (sound for subdir / sibling / nested / jail). This is the basis of
SL-152's **positional arming** (D3/D4): cd into
`<coord>/.doctrine/state/dispatch/spawn/` to arm, cd out to disarm
(self-clearing); supersedes the earlier "use SubagentStart to stamp" turn in
[[mem_019ec093bd7b71518489dd187b77f0f0]] and refines
[[mem_019ec65ecbc77282bad7e10a5240ad27]] (cwd placement) into an explicit
discriminator. Builds on [[mem_019efa04e19377c0938e58c059507a61]] (hook-replace
base control).

## P2 — the Agent return footer survives hook-creation

When a `WorktreeCreate` hook creates the worktree, the Agent return footer still
carries **`worktreePath:`** (plus `agentId:`). `worktreeBranch:` came back
`undefined` for a **detached** worktree (`git worktree add … HEAD`, no branch) —
so treat `worktreePath` as the **normative** datum and derive name/branch from it
(`name = basename(worktreePath)`), rather than depending on `worktreeBranch`.

## How to apply

For any doctrine claude-arm hook work: discriminate dispatch from benign spawns
by payload `cwd` position (orchestrator-controlled), carry only base out-of-band
in a file the hook reads, and read the worker location back from the footer's
`worktreePath`. Don't reach for `SubagentStart` stamping for base control — it
fires *after* `WorktreeCreate` and cannot feed base selection.
