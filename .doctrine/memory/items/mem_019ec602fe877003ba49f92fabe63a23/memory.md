# Claude Agent isolation:worktree base is session-root main, opaque — not the coordination-tree tip

**Empirical (SL-064/SL-066 dispatch, observed — not inferred).** A claude `Agent`
spawned with `isolation: worktree` creates its worktree forked from the
**session-root main HEAD region**, which is **opaque and not orchestrator-controllable**:

- It is **NOT** `origin/HEAD` (the earlier theory — disproven).
- It is **NOT** the orchestrator's isolated `dispatch/<slice>` coordination-tree tip,
  even when the orchestrator runs inside that tree. SL-064 §3's "orchestrator cwd ≡
  coordination HEAD ≡ B ⇒ worker forks B" assumption is **false**.
- `worktree.baseRef='head'` only changes the fork to *session-root local main HEAD*
  vs `origin/HEAD` — it **cannot** point the base at an arbitrary branch tip.

**Two distinct base problems:**
- **P1 — origin staleness:** worker forks `origin/main`, behind local main. `baseRef='head'`
  fixes this (forks local main HEAD). The original 32-commit SL-064 incident.
- **P2 — dependent-phase base unreachable:** a serial dependent phase needs the prior
  phase's code, which lives only on the isolated `dispatch/<slice>` branch, not on main.
  The Agent worker forks main → lacks it. **Unfixable** by `baseRef`; escapes are
  pre-audit integration to main (violates ADR-012) or fragile detached-HEAD games.

**Consequence (architecture):** serial-dependent phases are **not claude-dispatchable**,
and independent phases gain little. **Parallel dispatch belongs on the subprocess arm**
(codex/pi → DeepSeek via pi.dev, ~$1/hr): `doctrine worktree fork --base B --worker`
pins the worker to *any* explicit B — including the `dispatch/<slice>` tip — so it
handles **P1 and P2** by construction. `claude -p` subprocess is NOT an option (API-billed,
~$1000/hr). The claude (subscription, in-session Agent) arm is for **premium solo
`/execute`** in the coordination worktree; route parallel/dependent work to the cheap
subprocess arm. A `WorktreeCreate` creation-replacing hook *could* fix the claude arm
(fork orchestrator-chosen B) but costs the σ blast-radius (no `agent_type`/matcher ⇒
fires for every worktree subagent) — deferred escape hatch only.

Supersedes the mid-session memory that claimed the Agent worktree forks `origin/main`.
Related: [[mem.pattern.dispatch.fork-rung3-base-not-session-head]],
[[mem.pattern.dispatch.claude-agent-worktree-integrates-commit-onto-parent]],
[[mem.pattern.dispatch.spawn-backend-harness-agnostic-no-free-env-seam]].
