# Review RV-158 — reconciliation of SL-152

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit subject: **SL-152 PHASE-06** (secondary, droppable plugin-hook migration),
landed in two commits on `edge`: `c371b839` (phase 6a — plugin scaffold + hooks)
and `341867bd` (EX-4 — install delegates Claude hook-wiring to the plugin).
Self-audit, `reconciliation` facet. Reviewed surface: the **edge branch HEAD**
(authored arm, not dispatched — no candidate branch).

**Lines of attack — hold the phase to its (amended) exit criteria:**
- **EX-3** — both hooks (SessionStart `doctrine boot` + WorktreeCreate `doctrine
  worktree create-fork`) declared in `plugins/doctrine/hooks/hooks.json` with
  command form **bare `doctrine`** (NOT `${CLAUDE_PLUGIN_ROOT}`); `plugin.json`
  metadata-only; `settings.local.json` carries **neither** hook (mutual exclusion
  — double-wiring ⇒ double creation).
- **EX-4** — `doctrine install` (Claude) **stops** settings-wiring boot +
  create-fork and instead **prints** `/plugin marketplace add <repo>` + `/plugin
  install doctrine@doctrine`; `<repo>` from the `[install] repo` doctrine.toml key
  (default `davidlee/doctrine`). Both marketplace manifests retained
  (`.claude-plugin/marketplace.json` + `plugins/marketplace.json`).
  `install_claude_hook`/`HookSpec` retained as dead-code fallback. Codex/pi wiring,
  baseRef, skills, agent-def, boot-import ref UNCHANGED.
- **VT-1/VT-2** — install golden: post-install no SessionStart/WorktreeCreate hook
  in settings, instructions printed, marketplace lists the plugin.
- **VA-1/VA-2** — plugin form fires live on 2.1.181 (parity probe + pluginUsage).
- **Cross-cutting** — ADR-001 leaf layering for the new `install_config` module;
  the pure/imperative split (no IO in `install_config`); mechanical conformance.

**Invariants pinned:** mutual exclusion (exactly one hook form active); RSK-2
(plugin step additive/droppable, no scope creep into the primary); ADR-001
(`install_config` is a leaf — serde defaults only, no IO); behaviour-preservation
on the codex/pi arm; `PHASE-NN`/`EX-` ids immutable (amendments append).

## Synthesis

**Closure story.** PHASE-06 — the secondary, droppable plugin-hook migration —
landed its amended exit criteria (EX-3/EX-4, superseding the false EX-1/EX-2
premises via design D12). The phase moves doctrine's Claude integration from a
bespoke `settings.local.json` hook block to an idiomatic plugin: both session
hooks now live in `plugins/doctrine/hooks/hooks.json` and `doctrine install`
delegates wiring to the user by printing the marketplace+install commands rather
than writing global config.

**Verified green:**
- **EX-3** — `hooks/hooks.json` carries both `SessionStart → doctrine boot` and
  `WorktreeCreate → doctrine worktree create-fork`, command form **bare
  `doctrine`** (the `${CLAUDE_PLUGIN_ROOT}` premise was correctly rejected — D12
  Δ1: no executable ships in the plugin dir). `plugin.json` is metadata-only.
- **EX-4** — install stops settings-wiring boot + create-fork and prints
  `/plugin marketplace add <repo>` + `/plugin install doctrine@doctrine`; `<repo>`
  resolves from the `[install] repo` key (default `davidlee/doctrine`, confirmed
  correct by the User). Both marketplace manifests present
  (`.claude-plugin/marketplace.json` canonical + `plugins/marketplace.json`).
  `install_claude_hook`/`HookSpec` retained as `expect(dead_code)` fallback.
- **VT-1/VT-2** — `e2e_claude_install::install_wires_skills_agent_and_delegates_hooks_to_plugin`
  asserts no SessionStart/WorktreeCreate/SubagentStart hooks post-install +
  instructions printed. **Green.**
- **VA-1/VA-2** — plugin form fired live on 2.1.181 (pluginUsage
  `lastUsedNumStartups==numStartups==495`; boot.md regenerated, worktree
  provisioned) — recorded in plan VA-2.
- **Mutual exclusion** (the central invariant — double-wire ⇒ double creation):
  the create-fork settings-wire was removed from **both** entry points
  (`install.rs` run_forward_steps + `skills.rs` run_install); VT-2 proves a fresh
  install no longer re-adds it. The transient dup described in the VA-2 note was a
  pre-fix session state, not a standing defect.
- Full suite green and `cargo clippy` clean **in an isolated `CARGO_TARGET_DIR`**.

**Standing risk — the gate trap (environment, not a slice defect).** `just check`
against the shared jail target (`/home/david/.cargo/doctrine-target-jail`) reports
the install test RED with `worktree hook: wired` — a **stale binary** from the
concurrent pi-dispatch build, exactly the hazard `AGENTS.md` warns of. After
`just rebuild-stale` (or a clean isolated target) the test passes. Anyone
re-verifying this slice must rebuild stale first or read a false failure.

**Tradeoffs consciously accepted:**
- No `[[selector]]` on SL-152 (F-1) → no mechanical conformance signal; covered by
  manual diff for this close, tolerated rather than retrofitted on a closing slice.
- Plan EX-4 prose key name is stale (F-2) → corrected via per-slice direct edit in
  reconcile; design D12 already states the truth.
- ADR-001 layering edited in-commit (F-3) → aligned; living registry, no REV.

No unresolved blocker. The phase is conformant to its amended design; the ledger
is clean for the `audit → reconcile` move.

## Reconciliation Brief

### Per-slice (direct edit)
- **plan.md / EX-4 reference (F-2):** the EX-4 parenthetical cites `[claude]
  plugin-marketplace` as the marketplace key; the shipped key and design D12 Δ4
  are `[install] repo` (default `davidlee/doctrine`), confirmed correct by the
  User. EX-4's id is immutable — append a correction note to `plan.md` (or a D12
  cross-reference) pointing the key name at `[install] repo`. No code change.

### Governance/spec (REV)
- **None.** No spec or governance finding. ADR-001's `install_config = "leaf"`
  annotation (F-3) was disposed `aligned` — routine registry upkeep, no REV.