# Design sketch: subagent as dispatch orchestrator (two modes)

<!-- Authored 2026-07-04. Supersedes the earlier `coordinator-exemption-design.md`
     (that framed the exemption as the core; codex pass 1 + the owner's MCP insight
     demoted it to an opt-in knob). Seeds a future slice (scope backsolved later).
     Consumes `subagent-orchestrator-probe.md`. Decisions + threat framing locked
     with the owner 2026-07-04; pending codex pass 2 + governance ratification. -->

## Status & next (2026-07-04 handover)

**Where we are.** Design converged through 2 codex adversarial passes. Shape is
sound; the one empirical gate (RSK-225) is now **DISCHARGED** — see below. Open items
are execution-time only. No slice yet (scope to be backsolved). Artefacts are
auto-committed as doctrine state (`a8623e33`, `4dd994b8`).

**Read in this order:** `subagent-orchestrator-probe.md` (evidence) → this file →
IMP-253 (Mode B keystone) → RSK-225 (gate — now discharged) →
[[mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only]].

**GATE DISCHARGED (RSK-225, probed 2026-07-04, Claude Code 2.1.198).** An `mcp__*`
**write** tool (`mcp__doctrine__memory_record`) was driven live through **both** wall
verdicts on the claude arm and bypassed the wall both times: under **Reject** (Bash +
Write denied `worktree-jail: cwd-not-a-worktree`) the MCP write landed in primary
`.doctrine/`; under **Jail** (`isolation: worktree`, primary + `.git` RO to Bash) the
MCP write's `canonical_path` was in the **primary** tree (verified). Subprocess-arm
half confirmed architecturally: the claude MCP server is a stdio child of the harness
(unconfined); a subprocess worker's stdio MCP would be a child of the jailed `pi` and
inherit `bwrap --ro-bind / /` (no bypass; current pi workers carry no MCP). So **Mode
B may rest on MCP-mediated writes on the claude arm.** The residual RSK-225 risk (no
conformance lint pinning worker `tools:`; undecided arm-unification posture) stays open
as a Mode-B build task, not a design blocker.

**Then:** backsolve the slice(s). Proposed split — a *core* slice (Mode B mechanical
happy-path funnel + the confined-orchestrator drive-loop, keystone IMP-253) and a
*follow-on* Mode A knob slice. Governance: ADR-012 REV + ADR-011 D6 amendment,
ratified at reconcile.

**Owner steers locked:** C (two-mode) · constrained≈adversarial (no hard line —
injection collapses it; don't rest integrity on trusting the worker) · exfil accepted
for THIS open-source repo, tighten for others · ADR REV at reconcile · cwd-agnostic
exemption · GC not per-SubagentStop (resume collision).

---

## Problem

The SL-182 `PreToolUse` jail confines every subagent (`agent_id` present →
`Jail(cwd)` or `Reject`) because its deny input carries `agent_id` + `cwd` but not
`agent_type` — it cannot tell an orchestrator subagent from a worker. An orchestrator
needs privileged writes (commit / integrate / `.doctrine/` state) that live outside
any single cwd, so a subagent cannot currently drive the dispatch funnel. Enabling it
moves orchestration off the main thread → parallel/hierarchical dispatch (RFC-011).
Evidence: `subagent-orchestrator-probe.md` (Claude Code 2.1.198, live).

## The load-bearing harness fact

**The jail only mediates `Bash|Edit|Write`.** `src/worktree/pretooluse.rs` `decide()`
returns `PassThrough` for every other tool; `Reject`/`Jail` are consulted only inside
`decide_bash`/`decide_write`. So **the `Agent` tool and `Read` pass through unconfined
— even under `Reject`** (probe-witnessed: a `Reject`'d depth-1 subagent still spawned
a nested subagent and read files). This is simultaneously the enabler of Mode B and a
latent worker-escape (see §Security invariant).

**Two caveats that gated Mode B (RSK-225) — first now discharged:**
- **MCP-write passthrough — WITNESSED (2026-07-04, was inferred).** A `general-purpose`
  subagent drove `mcp__doctrine__memory_record` through **both** `Reject` and `Jail`
  and the write succeeded both times, landing in the primary tree outside the jail cwd
  (see the GATE DISCHARGED note in §Status). The design may now rest on MCP-mediated
  writes on the claude arm.
- **Arm divergence (strong fork) — still holds.** The passthrough is a **Claude-arm** property — the
  MCP server is a separate process reached via the harness, unconfined relative to the
  subagent's bwrap. On the **subprocess arm** (codex/pi), a worker's **stdio** MCP
  server is a *child of the jailed process* → inherits the jail (**no** passthrough).
  So Mode B's "unconfined MCP server" holds on the claude arm but **not** for stdio MCP
  on the subprocess arm. A **persistent (http/sse) MCP** server outside the jail could
  unify both arms — a deployment departure from stdio simplicity. **Be explicit about
  where unification (the ideal) is served vs where the arms diverge pragmatically**;
  do not assume the claude-arm shape generalizes.

## Threat model (reframed — the earlier version is retracted)

**Retracted:** "constrained-not-adversarial" as a *security boundary*. A
prompt-injected worker is an adversarial worker — there is no hard line between "doing
the wrong thing out of stupidity" and "out of poisoning" (owner, 2026-07-04). So the
design must **not rest the integrity of any privileged path on trusting the worker.**

**Retracted:** "blast radius = local corruption only." Network is *up* in the outer
nix jail (only ssh is absent, and that is a *nix* property in `flake.nix`, not
doctrine's). So a mis-privileged agent's worst case is **local destruction +
exfiltration**, and `git push` to a `file://`/path remote needs no network at all,
and a trusted actor later pushes the durable history a bad commit introduced. The
design **must not claim containment it does not provide.**

**Consequence:** the *default* must be threat-model-independent. A per-project knob
may relax it where the owner accepts the residual (this repo: open-source,
single-owner — exfiltration accepted; most projects should tighten).

## Decision: C — two modes

- **Mode B (shipped default) — MCP-mediated trusted writes; no exemption.**
- **Mode A (opt-in `[dispatch]` knob) — coordinator exemption; raw-Bash/git
  orchestration for single-owner repos that accept the escalation surface.**

---

## Mode B (default — validated on the claude arm, RSK-225 discharged): confined orchestrator, privileged writes via MCP

> **Validated (claude arm).** The MCP-write passthrough Mode B depends on is now
> witnessed (RSK-225 discharged 2026-07-04 — §Status). Mode B is claude-arm-shaped: a
> subprocess worker's stdio MCP inherits the jail, so the subprocess arm needs either
> the existing import dance or a persistent (http/sse) MCP outside the jail to unify.
> The residual open item is enforcement (pin worker `tools:`; conformance lint), not
> feasibility.

**A fully-confined orchestrator subagent drives the whole funnel** — spawn workers
(`Agent`, passes the wall), every privileged write via doctrine **MCP tools** (run in
the unconfined MCP server, already the sole-writer), read freely (`Read`, passes). No
tool is un-jailed; the MCP tool surface **is** the trusted-write-split, delivered by
doctrine's existing MCP architecture.

- **Placement.** Orchestrator runs in the coordination worktree → `Jail(coord-cwd)`.
  Its raw `Edit`/`Write` are confined to the coord tree (where the slice's authored
  `.doctrine/` state lives — allowed, inside cwd). Boundary-crossing *mechanical*
  writes (commit, one-shot import, reap sibling worktrees, record-boundary, lifecycle
  flips) go through an MCP tool.
- **Scope of Mode B — the mechanical happy path, NOT everything (codex-2 #5).**
  Retract "ALL privileged writes". Several funnel ops are interactive / journal-
  coupled and cannot become clean worker-invokable gated tools: `refresh-base`
  (stops on conflict for operator resolve, `dispatch.rs:800`), `candidate create`
  (parks a conflicted branch, `dispatch.rs:1128`), `integrate` (consumes the operator
  corpus-clobber allowlist + journals the judgement, `dispatch.rs:2064`), `candidate
  admit` (`dispatch.rs:1500`). These stay **report-and-halt to the human/main-thread**
  — exactly the existing "conflicts report-and-halt, never auto-merge" posture. Mode B
  automates the file-disjoint happy path; conflict-judgement escalates as today.
- **Worker self-commit** = the keystone MCP tool, **IMP-253**: a jailed worker calls
  `mcp__doctrine__worker_commit`; the server (unconfined) commits its delta with the
  funnel belts (prove gate + scope/`.doctrine`-`.claude` rejection + **exactly one
  commit on B: `parent(tip) == B`, non-merge** — codex-2 #3: *ancestry* "descended
  from B" is too weak, it accepts a `B→C1→C2→C3` stack on a resumed worktree). Kills
  the "ro `.git` → orchestrator imports the working-tree diff" dance; keeps linked
  worktrees (no clone switch).
- **What the belts buy — bounded blast radius, NOT preserved intent (codex-2 #4).**
  Softened from "threat-model-independent". No *escalation* surface (nothing is
  un-jailed; a gated tool cannot exceed its contract). But the belts are path/base/
  prove-bounded, **not intent-bound**: a poisoned worker can commit a semantically
  hostile change that is in-scope and passes `prove`. Intent-preservation stays with
  **review/audit and the human**, as it does today — Mode B does not claim to replace
  it. The honest claim is "no new escalation class", not "safe against a hostile
  worker's in-contract actions".
- **Cost — larger build than it sounds (codex-2 #1).** The dispatch mutations are
  **CLI-only orchestrator verbs today**; the MCP server currently exports only
  review/memory/onboard tools (`src/mcp_server/tools.rs`). Mode B requires building
  the dispatch MCP surface (commit / import / reap / record-boundary / lifecycle),
  each carrying `import`'s belts. Real work — the natural architecture, but not free.

### Security invariant (Mode B) — enforced by the agent-def `tools` allowlist, guarded by a lint

Because MCP tools bypass the `Bash|Edit|Write` wall, jail completeness now **depends
on the worker toolset being pinned**: a worker holds *only* gated write tools
(exactly `mcp__doctrine__worker_commit`) and nothing else writable.

**This is enforceable, not just documentation (refining codex-2 #2).** A subagent can
only call tools listed in its definition's `tools:` frontmatter (harness-enforced —
`dispatch-worker` today is `Read, Edit, Write, Bash, Grep, Glob`, no MCP). So the
worker gets `worker_commit` by *naming that one tool*, never a broad `mcp__doctrine`
grant. **Live footgun (codex-2 #2):** the doctrine MCP server already exports writable
`memory_record` / `memory_edit` (`src/mcp_server/tools.rs`) — so a worker granted the
server broadly could write `.doctrine/memory/**`, violating its source-only contract.
Mitigations: (a) grant workers the *specific* tool name only; (b) add a **conformance
lint on worker agent-defs** that fails if a worker `tools:` list contains any writable
MCP tool other than the gated commit tool (there is no such guard today — the gap
codex named).

---

## Mode A (opt-in knob): coordinator exemption — with codex pass-1 fixes

For single-owner repos that prefer raw-Bash/git orchestration over MCP-tool-ifying the
funnel. A new `dispatch-orchestrator` agent type is stamped `coordinator` at
`SubagentStart` (matches `agent_type`), read at `PreToolUse` → `Orchestrator`
(unconfined). Fail-open-to-escalation, so gated behind `[dispatch]
allow-coordinator-exemption` (default **false**) and only sound where the owner
accepts a standing local-destruction+exfiltration residual.

| Hook | Role |
|---|---|
| `SubagentStart` (unconfined, blocking, per-spawn every depth) | `agent_type == dispatch-orchestrator` → write stamp; else nothing |
| `PreToolUse` (unconfined; it *is* the jailer) | read stamp → `Orchestrator`; absent → existing `Jail`/`Reject` |
| GC | **not** per-`SubagentStop` (collides with auto-resume — codex #6); sweep at dispatch teardown + orphan-sweep at `dispatch setup` |

**Codex pass-1 fixes folded in (the original forge analysis was falsified):**

1. **`extra_rw` descendant gap (BLOCKER, codex #1) — the store was *not* unforgeable.**
   `validate_policy` (`jail.rs:372`) rejects only `/`, `main_root` ancestors, and
   `.git` — a descendant like `.doctrine/state/dispatch/roles` is an *allowed*
   `extra_rw`, so a worker with a broad grant self-stamps. **Fix:** reserve the
   role-store path in `validate_policy` — reject any `extra_rw` that covers it, same
   invariant class as `.git`. **Canonicalization caveat (codex-2 #6):** the reserved
   path and every `extra_rw` must be normalized in **one frame against the coordination
   root** before the containment test — today `load_policy` canonicalizes `extra_rw`
   via ambient process cwd (`pretooluse.rs:275`), so a naïve check passes tests and
   fails in prod on a relative/symlinked entry. Normalize both sides identically first.
2. **Bare-`agent_id` stale/reuse bleed (codex #3).** Docs promise only "unique
   identifier for the subagent," not cross-session/global uniqueness. **Fix:** key the
   stamp on `(session_id, agent_id)` and write the expected identity (session_id,
   agent_type) *into* the file; `PreToolUse` validates the reader matches. Also closes
   the "still a cooperative flag" critique (#4) — the stamp is now bound, not bare
   presence.
3. **Tool-surface (codex #5)** — same worker-allowlist invariant as Mode B; plus the
   orchestrator toolset is pinned (no writable MCP the exemption would un-gate).

**Honest residual (Mode A).** Worst case = local destruction + exfiltration (network
up; ssh-absence is nix's, not doctrine's; `file://` push needs no network). Even fully
patched, Mode A **un-jails a subagent**, so any *future* wall hole re-opens escalation
— a standing liability. Acceptable only under explicit per-project opt-in.

---

## Governance (broadened — codex #7)

Not just an ADR-012 topology REV. The change revises the **unconditional-confinement
premise** (SL-182), the **ADR-011 D6 risk calculus** the probe leaned on, and the
SL-182 security boundary. Vehicle: an **ADR-012 REV** (topology: a confined-orchestrator
actor class) **+ an ADR-011 D6 amendment** (the "P(hook failure)≈0 × jail-bounded
harm" framing no longer holds once network-exfil is admitted) **+ a note on the
SL-182 confinement ADR/spec** that Mode A punches an opt-in hole in the unconditional
wall. Ratified at reconcile (sanctioned amendment path, SL-181 A1); ADRs are
owner-locked VH.

## Open questions / review targets

- **OQ-1 — `arm-spawn` under a subagent spawner + `Jail`. CLOSED (codex-2, "yes
  mechanically").** `arm-spawn` writes under `.doctrine/state/dispatch/spawn` inside
  the coord worktree (`create.rs:194`, `dispatch.rs:464`); bwrap grants that worktree
  rw (`jail.rs:482`) and `Agent` stays pass-through (`pretooluse.rs:130`). Still
  worth one live confirmation at execute-time, but no design blocker.
- **OQ-2 — Mode B funnel-verb coverage.** Which funnel operations lack an MCP tool
  today (integrate, reap, record-boundary, lifecycle flips) and must gain one; do
  their belts survive relocation into a worker-invokable tool?
- **OQ-3 — worker MCP allowlist enforcement.** Is there a harness/doctrine mechanism
  to *enforce* (not just document) that a worker holds no un-gated writable tool?
- **OQ-4 — custom `agent_type` visibility at `SubagentStart`** (probe saw
  `general-purpose`; confirm a custom type name surfaces identically).
- **OQ-5 — depth-5 ceiling** vs hierarchical-orchestration ambitions.

## Follow-ups (named, not this work)

- **IMP-253** — the gated worker-commit MCP tool (keystone of Mode B; independently
  retires the CLAUDE.md import-the-working-tree-diff dance).
- **Mislabel hardening** — attest a worker's `agent_type` at spawn against the slice's
  phase plan, so a compromised orchestrator cannot silently un-jail a worker (Mode A).
- **Exfil tightening for downstream projects** — a network-egress wall in the outer
  jail (doctrine-level, not nix-level) for projects that need real adversarial
  defence; the default Mode B does not require it, Mode A repos may want it.
