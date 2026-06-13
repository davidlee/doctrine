# Dispatch spawn backend must be harness-agnostic; worker-identity cannot rely on a free env seam

When designing how an orchestrator spawns dispatch workers (and how worker-mode is
enforced), two harness facts are load-bearing:

- **`claude -p` is billed at Anthropic Console API rates, not the interactive
  subscription** — so fanning out N workers via `claude -p` is economically
  prohibitive. It is also a **harness-specific** command. It therefore **cannot be
  a required element of a skill**, and cannot be assumed as the spawn backend.
- **Claude's in-session `Agent`/Task tool has no env seam** — the orchestrator
  cannot set a per-worker env var (e.g. `DOCTRINE_WORKER`) through it. So any
  enforcement keyed on an orchestrator-set env var **degrades to nothing on the
  claude harness**, leaving worker-on-main fail-open.

**Why:** doctrine is harness-agnostic by design (works for claude / codex / pi by
construction). A spawn/enforcement design that only works where a free, env-seamed
subprocess exists (local `codex exec`, pi self-subagent) silently excludes claude —
the dominant harness — and the cost is invisible until billing or a fail-open guard
bites.

**How to apply:**
- No harness-specific spawn command (`claude -p`) as a *required* skill step. Record
  a harness-agnostic *contract* (provision → arm → spawn; binary emits the env
  block); each harness *templates* its own concrete spawn, and at least one
  template must not need a billed subprocess.
- Make the **disk marker** (not env) the *primary* worker-identity signal; treat an
  orchestrator-set env var as a codex/pi-only optimisation, never the sole guard.
- State the achievable enforcement altitude **per harness** (claude: marker-only
  unless a free env seam is found; codex/pi: full).

Surfaced as SL-056 `inquisition-2.md` Charge XIII (CRITICAL, thesis-breaking).
Related: `[[mem.system.coordination.concurrent-design-shared-main-worktree]]`,
`[[mem.pattern.dispatch.fork-rung3-base-not-session-head]]`.
