# Design SL-199: Confined subagent orchestrator drive-loop (Mode B capstone)

<!-- Reference forms: entity ids padded (SL-199, SL-198, ADR-012, ADR-011);
     doc-local refs bare — §A/§B/§C/§D, D-B1, OS1, R1. Status: REVISED post external
     review (codex GPT-5.5, 2026-07-04) — F1/F2/F3 verified against source, integrated.
     Locked: §A (discriminator + one-shot hook-consumed arm), §B (funnel — every
     committed-output tool commits server-side; scope is a HARD pre-commit gate;
     conclude-phase atomic), §C (agent-def + lint), §D (drive-loop), §E (verification),
     §6 (probe). Next: reconcile slice-199.md deltas → /plan. -->

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
`base` whenever either Fork condition is live. **The `base` file is the same
location for both arms** — `<root>/.doctrine/state/dispatch/spawn/base` — only the
*trigger* differs (positional cwd vs coord-root∧dispatch-branch); no second base
source, no divergent read path.

**Disarm discipline (confined) — one-shot, consumed in the hook (revised, ext-review
F4).** No cd-out self-clear, and *not* a disarm-after-return (a crash between spawn
and return would leave a stale `base` that force-forks the *next* benign spawn off an
old base — manufacturing a fake worker identity + stale jail record, per codex F4).
Instead the drive-loop writes `base`+`jail.toml` immediately before each spawn
(`dispatch arm-spawn --path .` from coord-root — cwd-safe), and the **create-fork hook
consumes (deletes) `base` atomically the moment it Forks** — so the arm is strictly
one-shot and cannot survive to mis-fork a second spawn even across an orchestrator
crash. Serial: unambiguous. Parallel batch: one shared `base` per batch (existing
single-slot semantics, [[mem.fact.dispatch.single-slot-arming-rendezvous]]) — hook
consumption there is a plan detail (per-spawn token vs batch slot).

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

Three tools (was four; the phase-conclusion pair merges — D-B1). **Every tool whose
output must live in committed `dispatch/<slice>` history commits server-side** — the
confined orchestrator can't reach coord `.git`, and the CLI arm's "orchestrator
commits next" assumption (dispatch.rs:714, import.rs:82-97) does not hold for it.

| Tool | Wraps | Args | Commits? | Returns |
|---|---|---|---|---|
| `dispatch_import` | `run_import` (fork arm) | `{slice, name}` → base=coord tip, fork=`dispatch/<name>` | **yes** (code) | `{coord_tip}` |
| `dispatch_conclude_phase` | `run_phase`(completed) **+** `run_record_boundary`, **atomic** | `{slice, phase, code_start, code_end, note?}` | **yes** (metadata, one commit) | — |
| `dispatch_reap` | `run_gc` (patch-id landed-oracle belt) | `{slice, name}` | no (worktree/branch delete) | — |

(A start-of-phase `in_progress` flip, if committed, rides a thin `run_phase` call —
plan detail; the load-bearing sequence is import → conclude → reap.)

- **D-B0 — committed outputs commit server-side; scope is a HARD pre-commit gate**
  (revised, ext-review F1/F2/F5). `dispatch_import` applies **then** commits (the
  unconfined server writes coord `.git`, as `worker_commit` does for a worker),
  returning the new `coord_tip` (feeds `conclude`'s `code_end` + the next phase's
  `B`). **Scope/undeclared is the existing HARD belt, not advisory:** `classify_import`
  already refuses undeclared paths (`Refusal`, import.rs:130-134) — SL-199 keeps that
  refusal, server-side, **before** apply/commit. A scope violation ⇒ nothing lands ⇒
  **report-and-halt** to the main thread. This is *why* "one seam, two doors" holds:
  the belt is preserved intact, not weakened to advisory. It also removes the trust
  inversion codex F5 flagged — integrity never rests on the confined orchestrator
  blessing an already-committed over-reach it cannot `git reset` away.

- **D-B1 — discrete where independent; atomic where jointly-consistent** (revised,
  ext-review F1/F3). `import` and `reap` stay discrete — genuinely independent ops,
  distinct belts/timing, each rides its `run_*` seam. But `phase_status(completed)`
  and `record_boundary` are **jointly-consistent committed metadata** that must not
  split across a crash (see D-B3), so they **merge into one atomic
  `dispatch_conclude_phase`** — flip + boundary + a **single** server-side commit.
  Still one seam per door (it composes `run_phase` + `run_record_boundary`, no forked
  logic — DRY intact); the merge buys crash-atomicity a downstream recovery belt
  can't match cleanly. *This partly reverses the earlier "all four discrete" position*
  — codex demonstrated four independent thin wrappers under-model the transactionality
  the confined arm needs. *Alternative considered (rejected):* keep all discrete + a
  hard "completed-without-boundary ⇒ refuse funnel progress" recovery belt — preserves
  full discreteness but tolerates a transiently-inconsistent committed window and adds
  a recovery path over a single atomic commit.
- **D-B2 — coord resolved server-side by slice-id; no caller-supplied path**
  (mirrors SL-198 X1). Resolver = `git worktree list --porcelain` (primary) →
  worktree on `dispatch/<slice>`. **Sibling of SL-198's worker-by-agent
  resolver — shared enumerate step lands in SL-198; SL-199 adds coord-by-slice.**
- **D-B3 — the clobber doesn't occur in coord context; the risk is a *missing* row**
  (revised, ext-review F3). The [[mem.pattern.doctrine.phase-complete-clobbers-boundary]]
  clobber is **suppressed** here: `set_phase_status`'s arm-guard skips solo-binding
  whenever a *live* `dispatch/<slice>` worktree exists (state.rs:534-543) — exactly the
  confined orchestrator's context. So flipping `completed` installs **no** degenerate
  row to overwrite. The real failure mode is a **crash between the flip and the
  boundary write** leaving *completed-without-boundary* (detectable as `Missing`,
  state.rs). D-B1's atomic `dispatch_conclude_phase` (flip + boundary + one commit)
  closes it directly: the phase is never `completed` in committed history without its
  true `(B, coord_tip)` boundary in the same commit.

**Trust posture.** Called by the confined orchestrator (not a worker), on the
coord tree it already governs. No new belts — the engine seams' belts come along.
The orchestrator's tool-surface is pinned by SL-198's conformance lint (the
`orchestrator` marker) — §C.

**OS → plan.** `run_phase` semantics under `path=coord` (cross-tree guard,
slice.rs:1046?); the exact commit provenance (author/message) for server-side
commits-on-behalf (ext-review F6); whether a separate `in_progress` start-flip tool
is needed or the flip rides `conclude` only.

### 5.C — `dispatch-orchestrator` agent-def + conformance lint (LOCKED)

**Agent-def** — `.claude/agents/dispatch-orchestrator.md`, modelled on
`dispatch-worker.md`:

```yaml
---
name: dispatch-orchestrator
description: Confined dispatch orchestrator — drives the funnel for one slice from
  inside its coordination worktree; nested-spawns workers, lands their deltas via
  doctrine MCP tools, reports conflict-judgement back to the main thread.
doctrine-role: orchestrator
tools: Read, Edit, Write, Bash, Grep, Glob, Agent,
  mcp__doctrine__dispatch_import, mcp__doctrine__dispatch_conclude_phase,
  mcp__doctrine__dispatch_reap
---
```
(Final funnel membership tracks §B/D-B1 — the three core tools shown, plus a start-
`in_progress` flip tool if plan finds one needed; the allowlist is whatever §B ships.)

**Two layers pin two different things** (why native writers + a marker coexist):
- **The wall** (runtime) bounds *where* raw `Edit`/`Write`/`Bash` land — coord cwd
  only, shared `.git` RO. Native writers are safe *because* the wall confines
  their blast radius.
- **The lint** (static) bounds the *MCP* surface — the only privileged
  cross-boundary door. Asserts the def's `mcp__*` tokens ⊆ the `orchestrator`
  allowlist = exactly the §B funnel set. Any other `mcp__*` (a second server, a bare
  `mcp__doctrine` grant, `worker_commit`) ⇒ lint fail. Stops a doctored orchestrator
  smuggling a second write channel.

**§C's delta over SL-198's lint.** SL-198 lands the lint mechanism + parses the
marker/`tools:` list + the `worker → {worker_commit}` allowlist row. §C adds one
row: `orchestrator → {the §B funnel tools}` (`dispatch_import`,
`dispatch_conclude_phase`, `dispatch_reap`, + any start-flip tool). Per STD-001 the
allowlist references the **same named constants** §B defines for the tool names — no
second literal. §C is data (a table row), not new parsing.

**Reads go raw; only git-writes go MCP.** The tight allowlist is sufficient
*because* the orchestrator reads slice/phase/dispatch state via the raw `doctrine`
CLI over Bash (in-jail, cwd-safe — `doctrine slice show --path .`, `dispatch
status`, corpus search from the coord tree's `./target/debug/doctrine`). Reads
never cross the wall, so they need no MCP grant. Only the **git-boundary writes**
(§B) do. **Soft coupling:** the main thread spawns via `Agent
subagent_type: dispatch-orchestrator`, referencing this `name:` from a
skill/prompt (not a Rust constant, unlike the worker's) — lower-stakes drift,
pinned by prose not a test.

**Placement contract — self-enforcing, no hook.** Unlike the worker, the
orchestrator is **not** forked by `WorktreeCreate`: it's a plain subagent spawned
into a **pre-existing** coord tree (created by `dispatch setup`), the main thread
having parked its persistent cwd there before the `Agent` spawn ⇒ `agent_id`
present + cwd = a linked worktree ⇒ `Jail(coord)`. So no `SubagentStart` provision
hook, no jail record for the orchestrator itself, and therefore **no
`name:`↔Rust-constant drift pin** (the worker needs one because its
`SubagentStart` matcher keys on `DISPATCH_WORKER_AGENT_TYPE`; the orchestrator is
matched by the `doctrine-role` marker, a free string the lint owns). Mis-placement
(spawned from primary cwd) fails loud — first raw `Bash`/`Edit` ⇒ `Reject`.

**OS → plan.** OS-C1: is the allowlist a **ceiling only** (prevent excess
privilege) or also a **floor** (reject a def *missing* `Agent`/a funnel tool)?
Lean ceiling-only — SL-198's grain; an under-equipped def fails loud at runtime,
not a security concern. OS-C2: confirm SL-198's lint iterates **all** marked defs
(a client project's second `orchestrator`-marked def is covered) vs. name-matching
one file.

### 5.D — The drive-loop (LOCKED)

**Scope line — the orchestrator owns the per-phase fork→land loop *into the coord
tree*; cross-tree/trunk ops stay main-thread.** `prepare-review` and `integrate`
write trunk (outside the coord jail, RO) — the confined orchestrator *cannot*
touch them. It drives ready phases, lands each into the coord branch, then
**reports-and-halts** to the main thread, which runs `dispatch sync --integrate`.
This is why the funnel is a tight three-tool set, not more: the trunk-facing verbs
are never the orchestrator's.

**Serial per-phase cadence (happy path):**

1. **`arm-spawn --path .`** (raw Bash, cwd = coord-root — cwd-safe; writes
   `base=B`+`jail.toml` into `.doctrine/state/dispatch/spawn/`, inside its own
   jail). `B` = current coord tip.
2. **Spawn nested `dispatch-worker`** (`Agent`, isolation:worktree). Confined-
   subagent cwd = coord-root ⇒ §A's confined arm Forks: `dispatch/<name>` at `B`,
   jail record provisioned. *This is the base-control vanilla `isolation:worktree`
   lacks* — the harness forks off session HEAD unless a `WorktreeCreate` hook
   overrides (dispatch-mechanics.md:39-42); §A's fork **is** that override, and the
   **fail-closed** provisioning seam (unlike the read-only `SubagentStart` stamp,
   mechanics:154-166).
3. **Worker self-commits** via `worker_commit` (SL-198) — one gated commit `C`
   (`C^==B`) on its own branch.
4. **`dispatch_import`** → apply **+ commit** server-side (D-B0), returns
   `{coord_tip}`. Scope violation ⇒ hard refuse ⇒ report-and-halt (nothing lands).
5. **`dispatch_conclude_phase`** — flip `completed` **+** record the true
   `(B, coord_tip)` boundary **+** one atomic server-side commit (D-B1/D-B3). The
   phase never reaches `completed` in committed history without its boundary in the
   same commit.
6. **`dispatch_reap`** — belt-gated by the **patch-id landed-oracle** (`git
   cherry`, mechanics:108-117): `run_gc` refuses to delete a fork whose patch isn't
   yet in coord history. Crash-proof, sibling-move-proof; inherited free.
7. **Disarm is automatic** — the create-fork hook already consumed `base` at step 2
   (one-shot, §A F4); no explicit clear. Next phase re-arms with the new `coord_tip`
   as `B`.

**Conflict-judgement → report-and-halt.** Any op needing main-thread/human
judgement — a red worker verify, a **hard import scope refusal** (server-side, not
an orchestrator bless), a `refresh-base`/`candidate`/`integrate` race
(mechanics:134-152, all
report-and-halt by design) — the orchestrator returns a **structured summary** and
stops. Never auto-merges, never self-unblocks a dirty-trunk refusal.

**Serial vs parallel: deferred to plan.** The serial loop is the design baseline;
a parallel batch reuses the same per-phase steps under the existing single-slot
arming rendezvous (one shared `B` per batch,
[[mem.fact.dispatch.single-slot-arming-rendezvous]]) — worker count / file-disjoint
batching is a plan concern, not a design fork.

### 5.E — Verification alignment

What evidence each part must add/change (mode per criterion — VT test / VA agent /
VH human):

- **§A — VT (pure, table-driven).** Extend `classify_create`'s existing table:
  positional cases unchanged → Fork (regression, must stay green); confined
  `coord_root ∧ dispatch-branch ∧ base` → Fork; benign `coord_root ∧ dispatch-branch
  ∧ ¬base` → Passthrough; `coord_root ∧ ¬dispatch-branch ∧ base` → Passthrough;
  `coord_root ∧ base ∧ bad-sha` → `BadBase`. Plus e2e: both Fork arms provision the
  jail record; **one-shot arm (F4)** — the hook consumes `base`, so a second spawn
  with no re-arm takes Passthrough (no double-fork off a stale base).
- **§B — VT (tool integration).** Per tool: parse args → resolve coord by slice →
  `run_*` → serialize. `dispatch_import` asserts **commit happened** (coord tip
  advanced, one non-merge commit) **and** that an **undeclared-scope delta is HARD-
  refused before any commit** (F2 — nothing lands, report-and-halt). `dispatch_conclude_phase`
  asserts flip+boundary land in **one** commit and that a crash-simulated abort leaves
  *neither* (atomicity, F1/F3). Server-side commit **provenance** (author/message) is
  asserted against the contract fixed at plan (F6). Belt regressions (import scope,
  reap landed-oracle) ride the existing `run_*` suites unchanged (behaviour-
  preservation gate).
- **§C — VT (lint).** Extend SL-198's lint suite: the sanctioned orchestrator def
  passes; a def granting an extra `mcp__*` / a bare `mcp__doctrine` / `worker_commit`
  fails, message naming the offender. A fixture def under `.claude/agents/`.
- **§D — VH + VA.** The end-to-end confined drive-loop is orchestration — witnessed
  live (VH, as the §6 probe was) that a real phase forks→lands→concludes→reaps under
  the confined orchestrator; the mechanical pieces (steps 4–6) are covered by §B VTs.
  **Recovery VT (F3, replaces the void overwrite test):** a phase left `completed`
  without a boundary (crash between — only reachable if `conclude` is *not* atomic) is
  detected as `Missing` and refuses funnel progress; with atomic `conclude` this state
  is unreachable, which the atomicity VT above proves.

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

**Doc + spec deltas (deliverables, not just ratification):** the shipped
`install/dispatch-mechanics.md` needs a **Mode B section** (the confined-
orchestrator arm: MCP funnel, `dispatch_import` folds the commit, reads-raw/writes-
MCP split) — it currently documents only the main-thread + pi arms. And check
tech-spec-021 (dispatch orchestrator process) for a **REQ delta** covering the
confined-orchestrator actor class. Both land in-slice; sized at reconcile/plan.

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
- **R4 — commit-on-behalf provenance (ext-review F1/F6).** The server now commits
  *metadata* (conclude) and *code* (import) on the confined orchestrator's behalf. If
  authorship/message provenance is unspecified, the committer-of-record blurs.
  Mitigation: fix the provenance contract at plan; assert it (§5.E).
- **R5 — arm crash window (ext-review F4).** Closed by design: the create-fork hook
  consumes `base` one-shot, so an orchestrator crash mid-spawn cannot leave a stale
  arm that mis-forks the next benign spawn. Residual: verify the hook's consume is
  atomic w.r.t. the fork (plan).

**External review provenance.** codex (GPT-5.5) hostile pass, 2026-07-04 — F1
(`record_boundary` uncommitted), F2/F5 (undeclared is a HARD belt, not advisory;
no post-commit bless), F3 (`set_phase_status` suppresses solo-bind in coord context;
real risk is a missing row), F4 (one-shot arm), F6 (verification) — all verified
against source and integrated above. Trunk-facing report-and-halt boundary survived.
