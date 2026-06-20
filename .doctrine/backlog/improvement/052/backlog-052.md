# IMP-052: Orchestrator post-spawn marker check: abort an unstamped worker fork

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Origin

SL-056 PHASE-05 worker-mode floor decision (`worker-mode-floor-decision.md`,
owner-locked Option **C** + this observability rider, SL-056 §6). C drops the
G2 fail-closed-on-marker-absent floor — the marker is now a **positive** worker
signal, and an unstamped claude worker (SubagentStart stamp-hook failure) is
*not* refused at the CLI write seam (ADR-006 D2a, re-amended). The funnel + the
bubblewrap jail bound its blast radius, but one risk remains **not from harm,
from silence**: if the stamp hook starts failing, an unstamped worker looks
signal-identical to a legit solo writer, so the failure can quietly become the
status quo and erode the marker's value with nobody noticing.

## What

The orchestrator is the trusted sole writer and — unlike the read-only
SubagentStart hook, which **cannot abort a subagent** (ADR-011 D6) — it *can*
refuse to proceed. After spawning a worker into a fork, the orchestrator
**verifies the marker landed** (cheap stat / `worktree status` against the fork
root) before handing the worker its task. Marker absent ⇒ **abort the worker
fork** (rip life support) with a loud receipt: *"worker fork unstamped —
SubagentStart stamp hook failed; not dispatching."* This backstops the
un-failclosable hook at the layer that **can** enforce, and makes stamp-failure
impossible to silently normalise.

## Why this layer (not a CLI floor)

The G2 fix tried to fail-close at the **CLI write seam** (refuse marker-absent
linked writes), which collided with D6a (solo-in-worktree direct-write) and
killed a real workflow. The orchestrator layer is strictly better: it gates at
**spawn time, behaviour-independent** (doesn't depend on the worker ever
attempting a write or landing where a guard looks), and it does **not** refuse
the legit solo agent (which has no orchestrator spawning it). Observability +
fail-closed without the D6a contradiction.

## Scope

- Dispatch funnel / `/dispatch` skill: add the post-spawn marker-presence check
  as a gate before task handoff (claude rung) — or at the `import` step at the
  latest (it already report+halts; this catches the *unstamped* condition
  explicitly rather than inferring it from a `.doctrine/` touch).
- Likely a small CLI affordance: `worktree status -p <fork>` already resolves the
  mode/marker (SL-056 PHASE-05, `ec81b5e`); the orchestrator can shell it. Confirm
  it exits legibly for the absent-marker case so the funnel can branch on it.
- Tie to ADR-011 D9 amendment (the claude weaker-class import-verify) and ADR-006
  D2a re-amendment (which names this rider as the non-silence mechanism).

## Not in scope

- Re-introducing the CLI fail-closed floor (explicitly rejected — Option C).
- Confining the worker to its worktree (raw-tree confinement — ADR-006 D2b /
  ADR-008, separate).

## Delivered-overlap note — SL-123 (2026-06-20, partial; do NOT close)

**SL-123** added the claude-arm **pre-funnel footer gate** (§5.4b): a missing
`worktreePath:`/`worktreeBranch:` footer halts before the funnel, and a
`verify-worker --branch` coherence check binds the footer to one worker state.
That partly delivers this item's intent — *abort an un-isolated / unstamped worker
before task handoff* — for the **claude rung**, at the import step.

It does **not** close IMP-052 (per SL-123 design §9): SL-123 ships a *prompt-cadence
+ import-time* gate on the claude arm only; this item's full intent is an
**orchestrator post-spawn, behaviour-independent** marker-presence gate at spawn
time (before any task handoff), and a small CLI affordance (`worktree status -p
<fork>`) to drive it. The spawn-time / cross-arm coverage remains open.
