# Notes SL-182: Claude-arm subagent write-confinement hooks

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

---

## ▶ STATE (2026-07-01) — design integrated, AWAITING `/inquisition` (codex)

Lifecycle: **design**. `design.md` written + internal adversarial pass integrated
(§10). No code yet. **Next step: `/inquisition` on `design.md` with codex
(GPT-5.5, default reviewer per CLAUDE.md).** Then integrate findings → `/plan`.
All `.doctrine/` committed; tree clean (CLAUDE.md M = User's `docs/claude` note;
the stray `mem.fact.dispatch.dispatch-branch-prefix-not-coord-unique` is SL-181's
trap, not this slice).

## HANDOVER — for the inquisition agent

Read in order: `doctrine slice show SL-182` (scope), then **`design.md`** (the
target), `doctrine backlog show RSK-014` (the proven probe this graduates),
`doctrine adr show ADR-008` (the confinement gap this closes) + ADR-006 (D2b /
sole-writer). Evidence/apparatus: `.doctrine/backlog/risk/014/probe-h1/`
(`results.md`, `pretooluse-wrap.sh`, `pretooluse-pathcheck.sh`). Recipe memory:
`mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement` (trust high,
verified). Proven flag set: `scripts/pi-spawn-confined.sh`.

**Verify hook claims against `docs/claude`** (local official-docs cache,
authoritative over web/haiku — per CLAUDE.md). Already cross-checked: plugin
`PreToolUse` supported (`plugins-reference.md:111-119`), matcher regex
(`:98`), no hot-reload → `/reload-plugins` (`:394`).

### The design in one breath

Graduate the proven two-wall confinement (`PreToolUse(Bash)` → nested bwrap rw-only
the worktree + ro-`/`; `PreToolUse(Edit|Write)` pathcheck `realpath ⊆ cwd`) from
probe bash scripts into a **Rust subcommand** `doctrine worktree pretooluse` (pure
`jail.rs` + thin `pretooluse.rs`), registered via the **plugin `hooks.json`**.
Per-worker jail policy (`extra_rw`+`network`) keyed by worktree name, provisioned by
`create-fork`. `.git` hard-ro → worker can't self-commit → claude `/dispatch` funnel
converges onto the pi arm's working-tree-diff import. Linux/bwrap only; fail-closed
when bwrap absent (macOS → IMP-045). **Path L** (linked worktree); the standalone-
clone alternative is **IDE-024**; selector-sourced write-allowlist is **IDE-025**.

### Where to push hardest (the soft targets)

1. **A7 keying model (freshly rewritten — highest risk).** The first draft keyed
   the policy by `agent_id` written by the orchestrator pre-spawn; that is
   IMPOSSIBLE (`agent_id` is harness-assigned at spawn). Now: orchestrator
   (`dispatch arm-spawn`) declares policy to a pre-spawn handshake location;
   `create-fork` (knows the new worktree `name`) provisions it to
   `<main>/.doctrine/state/dispatch/jail/<name>.toml`; PreToolUse resolves by
   `cwd → basename`. **Is this handshake actually race-free and is the pre-spawn
   declaration location real?** Does `create-fork`'s payload truly carry the name
   for the *claude Agent* spawn (vs the benign passthrough)? Probe finding 5 says
   benign spawns are detached-HEAD passthroughs — confirm the *armed* worker path.
2. **V-plugin bet (R2/D-reg).** The probe proved PreToolUse via `settings.local.json`,
   NOT the plugin path. Design *chooses* plugin (user steer: prior tests uniform).
   Docs confirm the event is supported, but **does it fire for a worktree
   subagent + honour `updatedInput` via the plugin path?** Unproven; gated as the
   first execute step. Is gating-not-proving acceptable at design-lock?
3. **Funnel convergence (objective 5 / R8).** ro-`.git` removes claude self-commit,
   breaking the `B..S` single-commit delta-check (`dispatch/SKILL.md:46`). Design
   converges to working-tree-diff import. **OQ-2: does the claude harness surface
   the worktree diff to the orchestrator when the worker's `git commit` is blocked
   RO?** Unverified harness behaviour (could the harness drop uncommitted changes on
   worktree collapse?). End-to-end gate, but is it a design-blocker?
4. **R7 orchestrator pass-through god-mode.** Accepted residual (OQ-5 deferred).
   Is "accepted" defensible, or must OQ-5 land with this slice?
5. **INV-5 shell-quoting** in `opaque_wrap` (paths with spaces/quotes) — injection
   surface; confirm the test pins it.
6. **D3 `.git` hard-ro** — is the `validate_policy` rejection of `.git`-touching
   `extra_rw` complete (symlink/`..` evasion of the reject)?

### Don't-lose / durable findings

- **Funnel discovery:** the claude `/dispatch` arm currently *expects a worker
  commit* (`dispatch/SKILL.md:46` delta-check), unlike pi. Confinement forces
  convergence — both arms onto working-tree-diff import. This is the real
  cross-cutting consequence of the slice.
- **Efficiency tradeoff (R8):** convergence imposes the pi arm's "can't trust
  worker green → orchestrator re-runs suite" cost on the claude arm. Deliberate;
  IDE-024 (clone + cherry-pick) is the efficiency recovery, prioritise on observed
  cost.
- **Existing hook machinery is all Rust subcommands** (`boot --emit`,
  `worktree create-fork`, `worktree marker --stamp-subagent`) installed via
  embedded `plugins/doctrine/hooks/hooks.json` (auto-discovered) — the seam this
  rides. `src/skills.rs:1024` install; `src/worktree/create.rs:295` create-fork
  handler; `src/boot.rs:1098+` settings hook merge (the fallback path).
- **Decisions locked:** D1 Rust subcommand · D2 per-worker policy (worktree-name
  key) · D3 `.git` hard-ro · D4 Path L · D5 single-sourced bwrap core + parity
  test · D6 schema (`extra_rw`+`network`, footgun-deny) · D-reg plugin hooks.json
  (gated V-plugin).
- **Touch-set (design-target selectors):** `src/worktree/{jail,pretooluse,mod,
  shared,create}.rs`, `src/dispatch.rs`, `.claude/skills/dispatch-agent/SKILL.md`,
  `plugins/doctrine/hooks/hooks.json`.
