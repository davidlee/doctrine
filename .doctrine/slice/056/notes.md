# SL-056 — implementation notes

Durable cross-phase facts harvested as phases land. Runtime progress lives in the
gitignored phase sheets (`phases/phase-NN.{toml,md}`); this file holds what must
survive a handover.

## PHASE-01 — G1+G3 governance gate (done)

**G1 — ADR-008 accepted.** Revised → `accepted` (was `proposed`). Three folds:
- §5.1 false-green/false-red evidence cluster folded into Context (validates D-B1
  empirically): shared-target false-green (touch+rerun), worktree-removal false-red
  (stale `CARGO_MANIFEST_DIR`), debug-vs-release timing.
- D-B2 sharpened to a **standing structural fact** — flake ro-binds
  `~/.cargo/bin/doctrine` ⇒ in-jail `cargo install` is structurally impossible ⇒
  **no install race because no install** (cannot regress).
- D-B3 re-scoped around the **nested-userns feasibility question (OQ-2)**, not cost:
  the gate is whether unprivileged userns creation works in-jail (`bwrap
  --unshare-user --ro-bind / / true`), since the outer bwrap may seccomp-block
  `clone(CLONE_NEWUSER)`. codex/pi-only; ro-binds the marker only, never
  `settings.local.json`. Also reframed: build isolation is a perf/false-green
  concern, **not a trust signal** — trust rides the marker + import belt.

**G3 — the spawn-interface ADR is `ADR-011`** (slug `spawn-interface-agnostic-contract`).
**Cite ADR-011 by this id in PHASE-03 (G2) and PHASE-04 (G4).** It states the
harness-agnostic **contract** (D1 marker identity, D2 orchestrator owns
fork-or-mark+provision+per-wt-env-emission) + the **per-harness altitude table**
(D3) + codex/pi enhancement layering (D4) + the charge-2 base-pinning residual (D5)
+ the **φ two-valued claude fail-closed cell** (D6, marked `proposed`) + the **σ
blast-radius confession** (D7). It **references ADR-006, does not amend it** (that
is G2/PHASE-03, post-spike).

**Status discipline:** ADR-011 stays `proposed` — its φ cell (D6) is O3-spike-
contingent. PHASE-03 (G2) firms the φ cell to the PHASE-02 spike outcome and is the
phase that promotes ADR-011 toward `accepted`.

**VH-1 gate:** PHASE-01 exit requires the **design owner to accept** the ADR-008
transition and the ADR-011 draft before PHASE-02 (the O3 spike) begins. **Accepted**
(ADR-008 accept + the two ADR-011 review fixes: D4 base-pinning overclaim dropped,
D2 env emission-vs-delivery distinction; commits 75570b6, 7da9907).

## PHASE-02 — O3 spike result (THE gate) — empirically RED for the WorktreeCreate fail-closed path

Probed the **live** Claude Code harness (temporary WorktreeCreate + SubagentStart
hooks, `isolation: worktree` Agent spawns; all artifacts cleaned up). The deployed
behaviour **contradicts the published hook docs** — verified empirically, as the
standing note mandates.

**WorktreeCreate (the design's fail-closed primary path) — RED:**
- Fires for an `isolation: worktree` Agent spawn; **mid-session settings-hook edits
  take effect without restart** (agent-def *registration* does NOT — a mid-session
  `.claude/agents/*.md` is absent from the Agent registry until a fresh session).
- **Replaces creation, fail-closed:** a no-stdout hook → `WorktreeCreate hook
  failed: no output` → creation aborted. The fail-closed *capability* is real.
- **Actual payload:** `{session_id, transcript_path, cwd, hook_event_name, name}`
  where `name` = the agent *instance* id (`agent-<hex>`). **NO `subagent_type`/
  `agent_type`, NO `worktree_path`, NO `base_path`/`ref`.** (The docs claimed all of
  these — wrong for this version.)
- Consequences: (1) **cannot gate on subagent_type** at WorktreeCreate; (2) **no
  target dir in the payload** → a `create-fork` cannot know where to create; (3)
  **no base/parent field → charge-2 session-HEAD residual STANDS** (no hook-time
  `base-moved` pre-refusal possible). The default-created worktree HEAD was *not*
  the orchestrator's HEAD — base is opaque + not orchestrator-controlled, sharpening
  charge-2. **So `run_create_fork`-via-WorktreeCreate as specced in PHASE-10/§4b is
  NOT buildable on this version.**

**SubagentStart (the design's O3-red fallback) — GREEN:**
- Payload `{session_id, transcript_path, cwd, agent_id, agent_type, hook_event_name}`.
  **`agent_type` present** (named, e.g. `general-purpose` — matcher-scopable to
  `dispatch-worker`); **`cwd` = the worker's worktree path**.
- → A SubagentStart hook **can provision + stamp the marker into `cwd`** (the
  worktree), scoped by `matcher: dispatch-worker`. Fires *after* creation ⇒ the
  **fail-open created-but-unstamped window** (the D6 O3-red altitude, verbatim).
- **σ note (better than the WorktreeCreate branch):** the SubagentStart `matcher`
  scopes the stamp to `dispatch-worker` cleanly ⇒ **no blast radius on benign
  isolated subagents** (they never hit the stamp hook). The WorktreeCreate
  matcher/else-branch question is **moot** — there is no usable WorktreeCreate
  gating at all.

**Untested (low marginal value):** the literal `dispatch-worker` agent_type through
SubagentStart (registry needs a fresh session) — but `general-purpose` already
proved the *mechanism*; a custom name uses the same `agent_type` field + matcher.
WorktreeCreate's RED is **schema-level** (no type/path fields exist), so name-
independent — a custom def changes nothing there.

**Verdict (feeds PHASE-03 G2 + ADR-011 φ firming + PHASE-10):**
- **φ cell → O3-RED row.** claude fail-closed via WorktreeCreate is **not
  achievable** on this version; the achievable altitude is **SubagentStart-stamp
  (fail-open window) → prompt-enforced**, accident-fenced + prompt-enforced, not
  malice-proof (ADR-011 D6 O3-red).
- **PHASE-10 pivot:** drop `run_create_fork`-via-WorktreeCreate as the primary;
  **`run_stamp_subagent` (SubagentStart, matcher-scoped) becomes the primary claude
  mechanism** + Claude default worktree creation + provision-at-SubagentStart. The
  WorktreeCreate create-fork is deferred until the harness payload carries
  type+path (or an IDE-004 channel lands).
- **charge-2 base-pinning residual STANDS** and is sharper than assumed (opaque base).
- This is **within the locked design** — D6 is explicitly two-valued and named the
  O3-red row; the spike merely *selects* it (and finds it harder-red than the
  optimistic arm). PHASE-03 (VH-1) encodes it into ADR-006 + firms ADR-011 φ.
