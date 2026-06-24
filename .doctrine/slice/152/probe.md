All five report items answered. Verdict below.

## Probe result: skinny payload is **genuine**, NOT a jail artifact

**1. Version.** `claude --version` = **2.1.187** (in-jail was 2.1.181; env `AI_AGENT=claude-code_2-1-185_harness`). Spread of 6 patches — behaviour identical across all.

**2. Literal payload** (native, outside jail):
```json
{"session_id":"823ea32c-…","transcript_path":"…/-tmp-probe/….jsonl",
 "cwd":"/tmp/probe","hook_event_name":"WorktreeCreate","name":"agent-ac3537d5405e82f6c"}
```
Present: `session_id, transcript_path, cwd, hook_event_name, name`. **Absent: `agent_type`, `base_branch`/base, `worktree_name`, `branch_name`, path** — identical to in-jail. Docs' rich payload not delivered.

**3. Hook fired** for programmatic `Agent(isolation:worktree)`, named subagent → **yes** (payload.json written).

**4. Base control** → **yes**. Worker landed `/tmp/probe/wt-out` at `7298f69` (c1 = HEAD~5), not native tip c6, not `.claude/worktrees/`. Hook fully replaced native creation, set base out-of-band.

**5. Matcher** → **does NOT scope**. Set `matcher:"cavecrew-investigator"`, spawned **general-purpose** — hook still fired (landed wt-out/c1, payload rewrote with its agentId). Fires regardless of agent. Jail parity.

### Native vs in-jail — every cell matches

| Finding                               | In-jail 2.1.181   | Native 2.1.187        |
| ------------------------------------- | ----------------- | --------------------- |
| Fires for `Agent(isolation:worktree)` | Yes               | **Yes**               |
| Hook replaces native creation         | Yes               | **Yes**               |
| Doctrine controls base (HEAD~5)       | Yes               | **Yes**               |
| Payload shape                         | Skinny (5 fields) | **Skinny (5 fields)** |
| Matcher scopes by agent_type          | No                | **No**                |
| `AI_AGENT=…_harness`                  | Yes               | **Yes (also native)** |

### Verdict
**Not the jail. The deployed harness genuinely emits the thin payload; docs are ahead of the build.** Hypothesis (bubblewrap / `_harness` build strips fields) **refuted** — native carries the same `_harness` AI_AGENT and the same five fields. 

**SL-152 holds** — design never depended on rich payload: hook is creator, sets base out-of-band, discriminates via orchestrator-dropped marker (not `agent_type`, which is absent), passes benign subagents. No design change needed.

Probe debris in `/tmp/probe` (6 commits, hook.sh, settings.json hook, wt-out, payload/env/log). Want me to tear it down, or leave for inspection?