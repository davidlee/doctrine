# IDE-004: Channels spawn backend for claude workers (env-seamed, subscription-tier, no -p)

Surfaced during the SL-056 Charge XIII `/consult` (keystone spawn-backend re-survey).

## Why

SL-056 concludes the claude harness has **no free env seam** for worker spawn:
`claude -p` is Anthropic-API-billed (not subscription) so fan-out is economically
prohibitive, and the in-session `Agent` tool exposes no env-passing parameter.
SL-056 therefore moves worker **identity onto a disk marker** (harness-agnostic)
and demotes env-arm to a codex/pi-only optimisation — claude tops out at
marker-only enforcement (no orchestrator-set env, no per-wt env injection, no
bwrap).

**Channels** may reopen the env-bearing claude path *without* `-p`. Booting with
`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` lets the orchestrator prompt an agent
launched without `-p` over a channel — a subscription-tier, in-session spawn that
(POC suggests) is viable and could carry an env seam. If it does, claude could
rejoin the codex/pi "full enforcement altitude" tier: orchestrator-set
`DOCTRINE_WORKER` (worker-on-main catch becomes universal again), per-worktree env
provisioning (the generalisable seam SL-056 defines, of which CARGO_TARGET_DIR is
the doctrine-repo-local consumer), and a wrap point.

## What

- Validate channels as a claude worker spawn backend: can the orchestrator set
  per-worker env (`DOCTRINE_WORKER`, project per-wt env) on a channel-launched
  agent that the worker's doctrine process reads via `var_os`?
- If yes: add a `claude-channels` template to the per-harness spawn router
  (SL-056's `/dispatch-*` sub-skill family) and lift claude's altitude-table row
  from marker-only toward full.
- Re-adjudicate the first-tribunal Charge III scope: env-as-worker-on-main-catch
  becomes universal again rather than codex/pi-only.

## Risks / caveats

- **Experimental.** Gated behind `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`;
  Anthropic could withdraw it from the subscription tier without notice. Any
  reliance must stay an *optimisation* layered over the marker-primary identity
  baseline — never a required element (per
  `[[mem.pattern.dispatch.spawn-backend-harness-agnostic-no-free-env-seam]]`).
- POC only so far (David). Needs a real propagation spike (the same gate SL-056
  Charge III demands for any env-bearing backend).

Ref: https://code.claude.com/docs/en/channels-reference
