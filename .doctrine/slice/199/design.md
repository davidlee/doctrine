# Design SL-199: Confined subagent orchestrator drive-loop (Mode B capstone)

<!-- Reference forms: entity ids padded (SL-199, SL-198, ADR-012, ADR-011);
     doc-local refs bare — §A/§B/§C/§D, D-B1, OS1, R1. Status: SHAPING.
     Locked: §A (create-fork discriminator), §B (MCP funnel surface), §6 (probe).
     Provisional: §C (agent-def + lint), §D (drive-loop). -->

## 1. Design Problem

Move dispatch orchestration **off the main thread** onto a fully-confined
`dispatch-orchestrator` subagent. It sits in `Jail(coord-cwd)`: raw `Edit`/`Bash`
reach the coordination tree (inside its cwd), but the shared `.git` is RO-walled,
so every boundary-crossing git write rides a **doctrine MCP tool** (witnessed to
bypass the `PreToolUse` wall — RSK-225 discharged). No exemption, no un-jailing:
the MCP tool surface *is* the trusted-write split. This is the serial-dependent
**capstone** of RFC-005 Mode B, building on SL-198's `worker_commit` keystone,
per-worktree jail record, and conformance lint. It unlocks off-main-thread and
parallel/hierarchical dispatch (the RFC-011 lever: main context stays a thin
cache-stable router; the ~40–49k/phase funnel ceremony burns a disposable
subagent context).

## 2. Current State (witnessed 2026-07-04, Claude Code 2.1.198)

- **The wall** (`src/worktree/pretooluse.rs`, `jail.rs::resolve_target`): every
  subagent carries `agent_id` ⇒ `Jail(cwd)` (cwd a linked worktree) or `Reject`
  (else). MCP tools bypass it (`decide()` mediates only `Bash|Edit|Write`).
- **create-fork** (`src/worktree/create.rs`, the `WorktreeCreate` hook): forks a
  worker at the armed base + provisions the per-worktree jail record
  (`jail/<name>.toml`) **iff the payload `cwd` == the arming spawn dir**
  (positional discriminator, `create.rs:362`); else Passthrough (detached, no
  record). The main thread satisfies this by parking its **persistent** Bash cwd
  in the arming dir.
- **MCP server** (`src/mcp_server/tools.rs`): exports review/memory/onboard only;
  `root` fixed to **primary**. Each tool = parse args → call the same `run_*` the
  CLI uses → serialize. No dispatch surface yet.
- **Funnel engine seams already callable** (clap-free, explicit params):
  `run_import` (import.rs:264), `run_gc` (gc.rs:221), `run_record_boundary`
  (dispatch.rs:718), `run_phase` (slice.rs). The dispatch mutations are **not**
  CLI-shell-coupled at the engine level — only unexposed via MCP.
- **SL-198 (needs, `ready`, not executed)** delivers: `worker_commit`, the
  enriched `jail/<name>.toml` record `{name,dir,branch,base,coord}` + its gc
  deletion, the coord-tree enumerate/probe resolver, the conformance lint (already
  handling the `orchestrator` marker for SL-199 reuse), the import-a-commit switch.

## 3. Forces & Constraints

- **RFC-005 threat model (locked).** No un-jailing; the design must not rest
  integrity on trusting a worker *or* the orchestrator. Belts are server-side.
- **ADR-012** — coordination worktree is the sole write target; no per-worker
  projection onto shared branches; integration-sync is a verb, conflict ops
  report-and-halt, never auto-land.
- **ADR-011 D6** — the `WorktreeCreate` payload is thin (no agent_type/base/path);
  positional/marker discrimination only. Mode B revises the D6 risk calculus
  (governance §7).
- **ADR-006 sole-writer / ADR-008 + SL-182 confinement** — Mode B punches no hole
  in the wall; SL-181 (Mode A exemption) is out of scope.
- **DRY / behaviour-preservation** — reuse `run_import`/`run_gc`/
  `run_record_boundary`/`run_phase` and SL-198's resolver; existing funnel +
  main-thread suites stay green unchanged.

## 4. Guiding Principles

Smallest coherent capstone over existing seams. The confined orchestrator does
*local* work raw (coord-tree edits, arming writes, `Agent` spawn) and every
*boundary-crossing* write through a thin MCP tool that rides an existing engine
belt. One seam, two doors — no forked funnel logic.

## 5. Proposed Design

### 5.A — create-fork Fork discriminator (linchpin, LOCKED)

**Why net-new.** A **jailed subagent's Bash cwd resets to its assigned worktree
root every tool call** (doc-confirmed `subagents.md:263`; no spawn-time cwd
override exists — full negative result in
[[mem.fact.dispatch.confined-subagent-cwd-resets-breaks-positional-arming]]).
So a confined orchestrator can only ever present **coord-root** as a nested
worker's `WorktreeCreate` payload cwd — never the arming dir. The positional
discriminator therefore always takes Passthrough: no `dispatch/<name>` branch, no
jail record ⇒ SL-198's `worker_commit` cannot resolve the worker. Witnessed live.

**Target — an additive second Fork trigger; positional path unchanged.**
`classify_create` (pure, table-tested) gains two inputs:

```
classify_create(cwd_resolved,
                cwd_is_arming_dir,   // main-thread positional — UNCHANGED
                cwd_is_coord_root,   // NEW: payload.cwd == resolved root
                coord_in_dispatch,   // NEW: root on a dispatch/<NNN> branch
                base, name) -> CreateAction
```

Fork iff `cwd_is_arming_dir` **(main-thread)** OR
`cwd_is_coord_root ∧ coord_in_dispatch ∧ base present/plausible` **(confined)**;
else Passthrough. Both arms run the same `act_on_create::Fork` → `fork_core` +
`provision_jail_policy` (no second impl).

**Shell gather** (create.rs ~362): `cwd_is_coord_root = (root == payload.cwd)`;
`coord_in_dispatch = git -C root branch --show-current =~ ^dispatch/[0-9]+$`; read
`base` whenever either Fork condition is live.

**Disarm discipline (confined).** No cd-out self-clear. The drive-loop writes
`base`+`jail.toml` immediately before each spawn (`dispatch arm-spawn --path .`
from coord-root — cwd-safe) and clears `base` after the worker returns. Serial:
unambiguous. Parallel batch: one shared `base` (existing single-slot semantics,
[[mem.fact.dispatch.single-slot-arming-rendezvous]]).

**The branch guard is hygiene, not a security boundary — forge-proofing not
required.** Satisfying `coord_in_dispatch` grants a spawn *nothing* a plain jailed
worker lacks — a forked worker is walled by `PreToolUse` and belt-gated at
`worker_commit` regardless of Fork/Passthrough. Failure modes are benign: a loose
guard at worst spuriously Forks a benign spawn (debris, not escape); a jailed
worker cannot spoof it (the spawn dir is outside its worktree = RO, shared `.git`
= RO, so it can neither arm nor create a `dispatch/<NNN>` branch — only the
trusted orchestrator arms, inside its own coord-root jail). Contrast the SL-181
coordinator marker, which *un-jails* and therefore *must* be forge-proof; this
discriminator lifts nothing, so a cheap branch-name check suffices. Real security
lives in the wall (confinement) + SL-198's server-side belts (blast radius),
neither of which depends on this guard being correct.

**Invariants / tests (pure TDD):** positional cases unchanged → Fork; confined
`coord_root ∧ dispatch-branch ∧ base` → Fork; benign `coord_root ∧ dispatch-branch
∧ ¬base` → Passthrough; `coord_root ∧ ¬dispatch-branch ∧ base` → Passthrough;
`coord_root ∧ base ∧ bad-sha` → `BadBase`. Both Fork arms provision the record (e2e).

**OS → plan.** OS1: is a coord tree *always* on `dispatch/<NNN>` (never detached
mid-op)? Verified live for SL-198/185; confirm `dispatch setup` never detaches —
if it can, the guard needs a state-file marker instead of the branch check. OS2:
keep-both vs retire-positional — recommend keep-both (lower risk, positional's
cd-out self-clear is a free safety; retiring is a larger behaviour change for no
confined-arm gain).

### 5.B — Dispatch MCP funnel surface (LOCKED)

Four **discrete** MCP tools, one per engine seam. Each: parse `{slice, …}` →
**resolve the coord tree server-side by slice-id** → call the existing `run_*`
with `path = <coord>` → serialize.

| Tool | Wraps | Args |
|---|---|---|
| `dispatch_import` | `run_import` (fork arm) | `{slice, name}` → base=coord tip, fork=`dispatch/<name>` |
| `dispatch_reap` | `run_gc` | `{slice, name}` |
| `dispatch_record_boundary` | `run_record_boundary` | `{slice, phase, code_start, code_end}` |
| `dispatch_phase_status` | `run_phase` | `{slice, phase, status, note?}` |

- **D-B1 — discrete, not one coarse `advance` tool.** One seam/two doors (DRY);
  each carries its engine's existing belts (import's `classify_import` scope +
  `S^==B` ride free); smaller blast radius; matches `worker_commit` grain.
  *Rejected* — a bundled `dispatch_conclude_phase`: fewer round-trips but merges
  distinct failure/halt semantics, diverges from CLI seams, harder to test.
- **D-B2 — coord resolved server-side by slice-id; no caller-supplied path**
  (mirrors SL-198 X1). Resolver = `git worktree list --porcelain` (primary) →
  worktree on `dispatch/<slice>`. **Sibling of SL-198's worker-by-agent
  resolver — shared enumerate step lands in SL-198; SL-199 adds coord-by-slice.**
- **D-B3 — lifecycle-flip must not clobber the boundary**
  ([[mem.pattern.doctrine.phase-complete-clobbers-boundary]]). Flipping to
  `completed` re-runs auto solo-binding (degenerate `start==end`). Drive-loop
  ordering: flip `completed` first, then `dispatch_record_boundary` with the true
  `(B, coord-tip)` range **last** — UPSERT-by-phase, last-writer-wins installs the
  true range. Locked into §D.

**Trust posture.** Called by the confined orchestrator (not a worker), on the
coord tree it already governs. No new belts — the engine seams' belts come along.
The orchestrator's tool-surface is pinned by SL-198's conformance lint (the
`orchestrator` marker) — §C.

**OS → plan.** `run_phase` semantics under `path=coord` (cross-tree guard,
slice.rs:1046?); whether `dispatch_import` returns the `undeclared` scope set for
orchestrator bless/reject.

### 5.C — `dispatch-orchestrator` agent-def + conformance lint (PROVISIONAL)

An agent-def listing `Agent` (nested spawn, depth-5) + the four funnel tools +
`Read`/confined `Bash`/`Edit`/`Write`, nothing else writable. Marked
`doctrine-role: orchestrator` so SL-198's conformance lint pins its `tools:` (no
writable MCP beyond the sanctioned funnel set). Placement contract: spawned with
cwd inside the coord tree (primary-tree cwd ⇒ `Reject`). *To shape.*

### 5.D — The drive-loop (PROVISIONAL)

Cadence per phase (serial happy path): `arm-spawn --path .` (write base+jail.toml)
→ spawn nested `dispatch-worker` (`Agent`, isolation:worktree) → worker
self-commits via `worker_commit` → `dispatch_import` → `dispatch_phase_status
completed` → `dispatch_record_boundary` (true range, last) → `dispatch_reap` →
clear base. Conflict-judgement ops (`refresh-base`, `candidate create/admit`,
`integrate`) **report-and-halt**: the orchestrator returns a structured summary to
the main thread. Serial vs parallel drive shape: deferred to plan. *To shape.*

## 6. Feasibility probe (empirical basis, 2026-07-04)

A confined `general-purpose` subagent (cwd = a linked coord tree → `Jail`) was
driven live:
- **Wall holds:** write inside coord tree OK; escape to `/workspace/doctrine`
  denied read-only. "No un-jailing" stands.
- **Nested `isolation:worktree` spawn works** from the confined subagent; forked
  at the armed base.
- **Positional arming broke:** the subagent's Bash cwd reset from the arming dir
  to coord-root between calls → `WorktreeCreate` payload cwd = coord-root →
  create-fork Passthrough → **detached, no branch, no jail record** (witnessed).
- **No override:** doc-confirmed inherent subagent behaviour; no frontmatter cwd
  field, no `Agent` cwd param, no hook-return lever
  ([[mem.fact.dispatch.confined-subagent-cwd-resets-breaks-positional-arming]]).

This is the empirical basis for §A. MCP-write bypass (the §B premise) was
witnessed prior (RSK-225, SL-198).

## 7. Governance (to ratify at reconcile)

Per RFC-005 §7: an **ADR-012 REV** (topology: the confined-orchestrator actor
class + the dispatch MCP funnel surface), an **ADR-011 D6 amendment** (the
positional-arming discriminator gains the confined-arm branch; the D6 risk
calculus under MCP-mediated writes), and a **note on SL-182/ADR-008** that Mode B
adds a *sanctioned MCP write surface* to a confined orchestrator without lifting
the wall (made safe by the conformance lint). ADRs are owner-locked VH.

## 8. Risks (initial)

- **R1 — OS1 (coord-tree branch assumption).** If a coord tree can be detached
  mid-op, the `coord_in_dispatch` branch guard misfires → benign spawns Passthrough
  when they should Fork (drive stalls, not a breach). Mitigation: verify at plan;
  fall back to a state-file marker.
- **R2 — MCP soft-dependency (load-bearing on this arm).** A confined orchestrator
  has no raw-git fallback; MCP-server health is a dispatch-stopper. Mitigation:
  main-thread dispatch remains the MCP-down fallback; document.
- **R3 — belt drift** between the funnel MCP tools and CLI verbs. Mitigation: call
  the same `run_*`; no forked copies (behaviour-preservation gate).
