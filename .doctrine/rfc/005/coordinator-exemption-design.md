# Design sketch: coordinator exemption — a subagent orchestrator under the SL-182 wall

<!-- Authored 2026-07-04. Design sketch seeding a future slice (scope backsolved
     later). Consumes the probe finding (subagent-orchestrator-probe.md). Threat
     model + decisions locked with the owner 2026-07-04; pending adversarial review
     (codex/GPT, multiple passes) + governance ratification (ADR-012 REV at
     reconcile). Companion to RFC-005; successor concern to SL-181 (which retired
     the *worker*-side positive marker — this inverts it to the *orchestrator* side). -->

## Problem

The SL-182 `PreToolUse` jail (`src/worktree/jail.rs::resolve_target`) confines
**every** subagent (`agent_id` present → `Jail(cwd)` or `Reject`), because its deny
input carries `agent_id` + `cwd` but **not** `agent_type` — it cannot tell an
orchestrator subagent from a worker. An orchestrator needs unconfined rw to the
shared `.git` (commit/integrate) and `.doctrine/` (state) — outside any single cwd —
so a subagent cannot currently drive the dispatch funnel. Enabling it moves
orchestration off the main thread and unlocks parallel/hierarchical dispatch
(RFC-011). Evidence: `subagent-orchestrator-probe.md` (live, Claude Code 2.1.198).

## Threat model (locked)

**Constrained-not-adversarial worker.** Doctrine defends against worker *accident*
(bad prompt, hallucinated write, wrong scope), not a worker *actively exploiting* to
escape confinement. Rationale: ADR-011 D6 (`P(hook failure)≈0 × jail-bounded harm`);
the human owns the machine and spawns every agent.

- **Explicitly documented, not assumed silently.** The design states the model so a
  future reader knows the exemption's integrity rests on it.
- **Push is NOT doctrine's guarantee.** Workers are *not* trusted to refrain from
  `git push`; push is denied because **ssh/network is absent in the outer nix
  bubblewrap jail** — a *nix* property, encoded in `flake.nix`, **not** in doctrine.
  The design must not claim to provide push-containment.
- **Adversarial defence = named later follow-up** (see §Follow-ups), not this work.

## Blast-radius property (why constrained is safe enough)

The exemption lifts **only the inner SL-182 dispatch jail**. An exempted subagent
still runs inside the **outer nix jail** (same process tree) → no ssh/network. So the
worst case of a *mis*-exemption (a worker wrongly un-jailed) is **local state
corruption** — it still cannot exfiltrate or push. Same blast-radius class as
ADR-011 D6's "lost local work." Defence-in-depth behind the constrained assumption.

## Decision D1 — exempt by a harness-stamped role, resolved at SubagentStart

A new agent type `dispatch-orchestrator` (tools include `Agent` + `Bash`; the
`dispatch-worker` type keeps **no** `Agent` tool). Role is resolved by the harness's
own `agent_type`, stamped at spawn, read at every tool call:

| Hook | Confined? | Role |
|---|---|---|
| `SubagentStart` | no (harness-child) | matches `agent_type == dispatch-orchestrator` → **write** stamp `<store>/<agent_id>`; else nothing |
| `PreToolUse` | no (it *is* the jailer; wraps the *subagent's* Bash) | **read** `<store>/<agent_id>`; present → `Orchestrator` (allow); absent → existing `Jail`/`Reject` |
| `SubagentStop` | no | **delete** `<store>/<agent_id>` (per-agent-death GC) |

`SubagentStart` fires once per spawn at every depth, blocking (probe: 3×), so the
stamp lands **before** the subagent's first command — no race window.

## Decision D2 — store location: runtime tier, outside every worktree cwd

`<primary-root>/.doctrine/state/dispatch/roles/<agent_id>` (presence-only file;
runtime/withheld tier, gitignored). Resolved by both hooks from the
`CLAUDE_PROJECT_DIR` anchor + `common_git_dir` (the code already derives `main_root`
this way in `resolve_provisioned_policy`). Load-bearing: it sits **outside any linked
worktree**, so a jailed worker (RO everywhere but its own cwd) cannot write it.

## Decision D3 — exemption is cwd-agnostic

`coordinator` stamp present → `Orchestrator` regardless of cwd — short-circuits
**both** the `Jail` and `Reject` branches. So the orchestrator runs from the coord
tree *or* the primary. "Mode, not location, decides" (ADR-006 D6a). The stamp, not
the cwd, is the trust source.

## Decision D4 — narrow exemption; nested-worker containment intact

The exemption keys on the **stamped role**, never on "spawned-by-a-coordinator." A
`dispatch-worker` spawned by an orchestrator gets **no** stamp → `Jail(cwd)` as today
(probe: depth-2 worker confined, wall is depth-agnostic). Confinement composes down
the tree.

## Forge analysis (the security core — clears SL-181's bar)

SL-181 retired the *worker* marker because a cooperative flag is forgeable and
confinement covered the case anyway. This is the **inverse** marker (identify the
*orchestrator* to un-jail it), so a forged stamp is **fail-open-to-escalation** —
higher stakes. Its integrity rests on two things SL-181 *blessed*, not on trusting a
flag. Three forge vectors, all shut:

1. **Self-stamp the store** — blocked. Store is outside every worktree cwd; a jailed
   worker's Bash (bwrap rw-only-cwd) and Edit/Write (pathcheck ⊆ cwd) are RO there
   (witnessed `D2_GIT=FAIL`, `D2_PRIMARY=FAIL`).
2. **Spawn an `orchestrator`-typed child to escalate** — blocked. `dispatch-worker`
   has no `Agent` tool; `agent_type` is set by the *spawner*, unforgeable by the
   spawnee.
3. **Race the stamp** — no window. `SubagentStart` blocks until the stamp lands,
   before the worker's first command.

Default fail-closed preserved: no stamp → confined. Only an explicit harness-typed
coordinator spawn lifts it.

**Residual trust (constrained model):** the orchestrator spawns workers with the
`dispatch-worker` type. A mislabel (worker spawned as `dispatch-orchestrator`) would
un-jail a worker — but that is orchestrator *correctness* (enforced by the arm
tooling / dispatch skill), not a worker-driven escalation. Hardening it against a
compromised orchestrator is the adversarial follow-up.

## Governance

The SL-182 wall is deliberately **unconditional** ("`PreToolUse` unconditionally
active on the claude dispatch path"). This exemption introduces a **second unconfined
actor class** → a topology change against **ADR-012**. Vehicle: an **ADR-012 REV**,
authored and **ratified at reconcile** (not hand-edited; the sanctioned amendment
path, per SL-181 A1). ADR-006 D2a/D2b get companion notes (the exemption is the first
*intentional* markerless-but-trusted actor besides the main thread).

## Open questions / review targets

- **OQ-1 — `create-fork`/`arm-spawn` under a subagent spawner.** The probe used raw
  `isolation:worktree`, not doctrine's `create-fork` hook (deferred, ADR-011 D6).
  Confirm the positional `arm-spawn` discriminator (cwd == spawn dir) still fires
  when the *spawner* is a subagent, and that an exempted orchestrator can run
  `arm-spawn` unconfined. (Empirical, execute-time.)
- **OQ-2 — stamp durability across the agent's life.** If `SubagentStop` misfires
  (crash), a stale `coordinator` stamp for a dead `agent_id` lingers. `agent_id` is
  unique per spawn, so a stale stamp cannot be inherited by a new agent — but add a
  sweep at `dispatch setup` as a backstop.
- **OQ-3 — role store concurrency.** Parallel orchestrators + workers write/read the
  store concurrently; presence-only files keyed by unique `agent_id` are race-free by
  construction, but confirm no directory-creation race.
- **OQ-4 — does `SubagentStart` matcher reliably see the custom `agent_type`?**
  (Probe saw `agent_type: general-purpose` in the payload — confirm a *custom* type
  name surfaces identically.)
- **OQ-5 — depth-5 ceiling** vs hierarchical-orchestration ambitions.

## Follow-ups (named, not this work)

- **Adversarial-worker hardening** — if the threat model ever tightens: keep the
  orchestrator's trusted writes (import/commit/integrate) *out* of the subagent — a
  confined orchestrator *drives*, a trusted `SubagentStop`-style capture hook
  *performs* the privileged writes (SL-182's capture seam, inverted). Large; deferred.
- **Mislabel hardening** — attest the worker `agent_type` at spawn against the slice's
  phase plan, so a compromised orchestrator cannot silently un-jail a worker.
