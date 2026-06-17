# Implementation Plan SL-088: Consolidate installer commands: single DWIM install with per-agent opt-in

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases turn the locked design (RV-057 reconciled) into a shipped
consolidated `doctrine install` that does base files + prompted forward steps
(memory, boot, per-agent skills) in one command. `doctrine claude install` and
`doctrine skills install` are removed outright — no deprecation period (no
external users). The underlying machinery (`corpus::sync_corpus`,
`boot::wire`, skills install) is preserved — now orchestrated from
`install::run()`.

## Sequencing & Rationale

**PHASE-01 — CLI surface first.** The skeleton must exist before anything
plugs into it. Moving flags, removing dead commands, and updating goldens is
a compile-checkable shape change with no behavioural depth. Doing it first
lets subsequent phases wire into `install::run()`'s real signature, not a
stub.

**PHASE-02 — orchestration next.** `detect_agents()`, `prompt_step()`, and
the forward-step dispatch are the heart of the consolidation. PHASE-01 gave
it the CLI shape; PHASE-02 fills it with behaviour. The per-agent skill
install functions are extracted from `skills.rs` here so PHASE-03 can
generalise.

**PHASE-03 — agent-def generalization.** A single `install_agent_def()` that
handles both Claude (flat canonical, grandfathered per design §4) and pi
(namespaced canonical). Rides PHASE-02's per-step dispatch — the skills
step now calls `install_agent_def()` for each agent. Depends on SL-084's
embed content existing at `install/agents/pi/dispatch-worker.md`.

**PHASE-04 — docs + e2e last among behaviour phases.** README.md and e2e
tests describe the shipped mechanism — they can't be correct until the
mechanism works. The e2e rewrite preserves the same assertions (hook merge,
agent def, skills) but drives them through the consolidated surface.

**PHASE-05 — gate.** Conventions gate runs once all code is in. Stale
`.doctrine/agents/claude/` directory removed (agent defs now installed from
embeds). Final status transition to `done`.

### Why not parallel?

- PHASE-02 through PHASE-04 all touch `install.rs` and `skills.rs` in
  overlapping regions — serial avoids merge conflicts.
- PHASE-04's e2e tests depend on PHASE-01 through PHASE-03 shipping correct
  behaviour to assert against.
- PHASE-03 depends on PHASE-02's per-step dispatch to call
  `install_agent_def()`.

## Notes

- RV-057 findings F-1 (README) and F-2 (e2e) are addressed in PHASE-04.
- The stale `.doctrine/agents/claude/` removal (RV-057 finding 1 nit) lands
  in PHASE-05.
- `just check` (fast, root pkg only) is the inner-loop gate; `just gate`
  (full workspace clippy) runs before each commit.
- No dependency edge on SL-084 needed — the embed path is known; this slice
  reads it, doesn't author it.
