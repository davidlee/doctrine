# Notes SL-182: Claude-arm subagent write-confinement hooks

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

---

## ▶ STATE (2026-07-01) — INQUISITION COMPLETE (RV-200), design returns for reconcile

Lifecycle: **design**. `/inquisition` run against `design.md` (codex GPT-5.5 +
inquisitor pass) → **RV-200: 10 findings, 3 BLOCKERS**. Design is NOT clean — it
must be reconciled before `/plan`. The three blockers hold SL-182's close-gate
(9 charges deliberately left answered-not-verified; F-9 acquitted/terminal).
**Next step: reconcile design.md against RV-200** (two blockers carry remediation
OPTIONS needing a User/`/design` steer), THEN `/plan`. Full verdict in
`doctrine review show RV-200` → `## Synthesis`. Tree: as before (CLAUDE.md M;
SL-181's stray mem trap not this slice).

### RV-200 verdict (the heresy)

- **F-1 (blocker)** per-worker custom policy is UNBUILDABLE through the single-slot
  arming rendezvous (`arm-spawn` = one shared `base`; `dispatch-agent` allows N
  parallel spawns/arming). Cut to strict default floor (rec) or serial-scope it.
  → couples F-4 (D2 §7 + authored scope still say `agent_id` keying §5.3 repudiated).
- **F-2 (blocker)** installer fails OPEN: bare-PATH plugin exec + only `exit 2`
  blocks (hooks.md:629-643) ⇒ stale/missing binary runs UNCONFINED (RSK-014
  reopened). §5.1/D1 (resolve_exec) contradicts §5.4 (bare PATH). Fail closed:
  absolute resolved exec or a shim that `exit 2`s on not-found.
- **F-3 (blocker)** funnel convergence rests on doc-DISFAVORED teardown:
  `WorktreeRemove` auto-`git worktree remove`s the subagent worktree on finish,
  NO decision control (hooks.md:2442/680/814) ⇒ uncommitted diff destroyed;
  "identical on both arms" is FALSE (pi orchestrator owns lifecycle, claude harness
  doesn't). Name a contingency: snapshot `git diff` in WorktreeRemove/SubagentStop
  before removal (rec), or Path C/IDE-024, or defer ro-`.git`.
- **majors** F-5 V-plugin fallback forbidden→make D-reg conditional, fallback
  same-phase · F-6 Edit/Write wall matches UNDOCUMENTED `NotebookEdit`/`notebook_path`
  (drop or pin schema first).
- **minor/nit** F-7 `network=true` default vs §4 "strictest floor" wording · F-8
  policy file's false "ancestor" rationale (ro-ness is `--ro-bind / /`) · F-10 §10
  understates doc coverage (agent_id hooks.md:595, updatedInput :818 ARE doc'd).
- **F-9 ACQUITTED** R7 orchestrator pass-through residual is defensible — agent_id
  harness-stamped present-iff-subagent (probe), worker can't forge absence; OQ-5
  deferral sound. Soft-target-4 answered: accepted, not must-land.

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

### Durable harness gotchas confirmed by RV-200 (→ `/record-memory` candidates)

Verified against `docs/claude` (authoritative cache), high confidence:

- **PreToolUse hooks fail OPEN.** Only `exit 2` blocks a tool call; ANY other
  non-zero exit (incl. command-not-found 127 from a missing/stale binary) is a
  NON-blocking error and the tool PROCEEDS (`docs/claude/hooks.md:629-643` + the
  Warning). A hook meant to enforce confinement MUST resolve to a guaranteed-present
  absolute binary or use a shim that `exit 2`s on exec failure — bare-PATH exec is
  not fail-closed. (Exception: `WorktreeCreate`, where any non-zero aborts.)
- **`WorktreeRemove` auto-destroys an `isolation:worktree` subagent's tree on
  finish.** Fires when the subagent completes; Claude runs `git worktree remove`
  automatically; the hook has NO decision control and failures are debug-log-only
  (`hooks.md:2442`, `:680`, `:814`). Uncommitted worktree changes are LOST unless
  snapshotted before removal. Consequence: a harness-owned worktree (claude Agent
  arm) is NOT lifecycle-equivalent to an orchestrator-owned worktree (pi/subprocess
  arm) — any "import the worker's diff" cadence must capture before teardown.
- **Single-slot arming rendezvous can't key per-worker state.** `dispatch arm-spawn`
  writes ONE shared `base` file per arming dir; `dispatch-agent` issues N parallel
  spawns off one arming (all read the same B). The harness-assigned worktree `name`
  exists only at create-fork, not pre-spawn — so any per-worker pre-declared state
  through the arming dir is batch-shared, not per-worker. (Dispatch design fact.)
