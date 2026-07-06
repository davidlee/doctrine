# Design SL-206: Workflow-templated slice-driver

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Drive a slice's phases to completion from a dynamic **Workflow** (Claude Code's
JS orchestration harness), not ad-hoc `Agent`-tool spawns. The RFC-011 lever:
main context stays a thin cache-stable router; each phase's ~40–49k funnel
ceremony burns in a **disposable** orchestrator+worker context; the Workflow
**`budget` pool** meters the whole spawned tree **live** — the capability ad-hoc
spawns lack (they give receipts only at the return boundary, no live cross-agent
pool).

Downstream of SL-199 (confined `dispatch-orchestrator` + the
`dispatch_import`/`dispatch_conclude_phase`/`dispatch_reap` MCP funnel).
De-risked by CHR-039 (probe matrix; `.doctrine/rfc/011/chr-039-findings.md`).

## 2. Current State

- **Funnel tools already return typed outcomes** (`src/mcp_server/dispatch.rs`):
  `FunnelOutcome::{Imported{coord_tip}, Concluded{coord_tip}, Reaped{fork},
  Refused{reason,detail}}` — `reason` a closed vocab. So a *funnel-tool-level*
  receipt largely exists; what is missing is the **phase-level** aggregate.
- **`dispatch status` rollup** (`src/dispatch.rs:2896`) reads per-phase status
  from the **gitignored runtime sheet** (`phase_rows`), plus slice-global
  aggregate logic (`all_completed`, `review_tip`, `bundle_stale`, guidance,
  rendering). Boundary rows (`src/boundary.rs` `BoundaryRow`, landed by
  `ledger::record_boundary`) are a **separate, committed** truth source.
- **Readiness authority** is `compute_next_phases` / `dispatch plan-next`
  (`src/dispatch.rs:2751`), not the status table.
- Orchestration today is ad-hoc `Agent`-tool spawns from the main thread; no
  live token metering across the spawned tree.

### CHR-039 empirical ground (verified live; do not re-derive)

A workflow `agent(isolation:'worktree')` fires the doctrine `WorktreeCreate`
hook (payload cwd = session dir, controllable); isolated forks reach doctrine
MCP with a clean binary (no isolation limit); a background worker has **no live
escalation channel** (human-in-loop is return-value only); `budget` plumbing is
live but `total=null` without a `+Nk` directive. **Safety posture: a workflow
spawns a confined orchestrator, never a bare worker** — containment is by policy
(orchestrator-sole-writer ADR-006, gated `worker_commit`, narrow MCP grants),
not by MCP-unreachability.

## 3. Forces & Constraints

- **ADR-006** — orchestrator is sole writer; the workflow script writes nothing.
- **ADR-011** — harness-agnostic spawn; the driver is one harness (Workflow)
  atop the shipped confined-orchestrator interface.
- **ADR-012** — dispatch topology (coord tree, class-routed projection).
- **Behaviour-preservation gate** (AGENTS.md) — refactoring shared machinery
  (`run_status`) must leave existing dispatch suites green **unchanged**.
- **STD-001** — no magic strings; tuning values are named constants.
- Workflow scripts have **no shell / no filesystem / no MCP** — cannot call
  doctrine directly; every doctrine call routes through a spawned agent.
- **IMP-174 unresolved** — the fork-authored-vs-edge split-brain has no shipped
  reconciliation; SL-206 must not paper over it.

## 4. Guiding Principles

- **Rust-primary, narrow.** The only new Rust is **three read-only** doctrine
  tools — the phase-receipt emitter (`dispatch_phase_receipt`), the readiness
  surface (`dispatch_next_ready`), and the divergence probe
  (`dispatch_authored_divergence`) — plus the conformance-authority update they
  force (§5.4/F1). The `/drive-slice` script is an install-templated reference,
  not `cargo test`-able. **No new write surface.**
- **DRY over the readiness/status authorities** — no parallel state machine in
  JS; consume `compute_next_phases` and a single per-phase projection.
- **Durable truth over disposable truth** — receipt status is sourced from the
  committed boundary ledger, not the gitignored sheet.
- **Containment by grant, not by hope** — the safety boundary is the subagent
  tool allowlist, checked (static + runtime), not assumed.
- **Report-and-halt** — the driver never crosses the authored split-brain and
  never auto-merges (SL-199 §D intact).

## 5. Proposed Design

> **↻ RE-OPENED 2026-07-06 — unjail model now current (POC landed).** The prior
> workflow-spawn / confined-orchestrator model is RETIRED. PHASE-07 proved a
> Workflow `agent()` leaf cannot drive committing dispatch (three walls: no
> `Agent`, RO `.git`+no `DispatchRecord`, `worktree-jail` deny); the POC then
> proved the replacement — a **durable workflow loop** spawning **alternating
> ephemeral nominated-unjailed orchestrators + jailed workers**, escalation closed
> at the `PreToolUse(Agent)` spawn seam (P1/P3/P4, unjail-direction.md §6). §5.1
> below is the current system model; §5.6 the security posture; the retired
> confined-placement delta is struck from §5.4. Reasoning chain + evidence:
> **[`unjail-direction.md`](./unjail-direction.md)**; `notes.md` FINDING 4/5 + the
> topology-REFINED section.

### 5.1 System Model

The workflow is the **durable serial loop and the sole spawn authority**; it
**alternates** *ephemeral* **nominated-unjailed** orchestrators and **jailed**
workers. Crucially there are **never two orchestrators back-to-back**: each
*interior* orchestrator does **two jobs in one agent** — dispose the previous
worker's commit *and* prep the next worker's context — so an orchestrator always
sits *between* two workers (the first preps-only, the terminal disposes-only).
Spawn authority (workflow) is deliberately **separated from import authority**
(the disposing orchestrator) — a genuine departure from shipped `/dispatch`,
forced by wall #1: a workflow leaf has no `Agent` tool, so an orchestrator leaf
cannot spawn the worker; only the workflow can.

```
Workflow /drive-slice   (JS; args:{slice}; durable serial loop; SPAWN AUTHORITY)
  bootstrap: O₀ preps-only (no prior worker to dispose) → next_ready + PREP phase-1 worker
     claude arm: O₀ = agent(dispatch-probe)  ── READ-ONLY, UNPRIVILEGED, NO nomination
                 (base_B from dispatch_phase_receipt.dispatch_tip; prompt from Read/Grep) ─┐
     pi arm:     O₀ = agent(dispatch-orchestrator) ── nominated-unjailed (needs `git worktree add`)
  loop, ALTERNATING O and W — one orchestrator BETWEEN two workers, TWO jobs:

    W_i  = worker (JAILED): claude arm = workflow agent(dispatch-worker, isolation:worktree)
                            pi arm     = already spawned by the prior O_{i-1}
       · edits source in its fork; self-commits via worker_commit MCP  [claude, TARGET 2b-B]
         (pi / fallback (A): leaves an uncommitted diff, the disposing O imports it)
       ▶ { fork_tip, summary }                                        → workflow holds

    O_i  = agent(dispatch-orchestrator) ── NOMINATED-UNJAILED ── TWO JOBS in one agent:
       (1) DISPOSE W_i — import delta onto B_i, verify, commit ONE on dispatch branch,
           dispatch_conclude_phase→boundary, dispatch_reap; on a FIXABLE defect emit a
           fixup verdict instead → workflow revives W_i on its fork (bounded, §5.5)
       (2) PREP W_{i+1} (iff W_i accepted ∧ next_ready non-empty) — capture B_{i+1},
           distill the next worker prompt; pi arm → git worktree add + spawn subprocess
       · SubagentStart(dispatch-orchestrator) → allowlist → PreToolUse PassThrough (§5.6)
       ▶ HopReceipt { dispose: receipt_status|verify|fixup|halt · prep: next ctx · next_ready }

    JS: hop.fixup → revive W_i; hop.halt → report+stop; else advance on hop.prep (null ⇒ done)
  → O₀ preps-only, the terminal O disposes-only, every interior O does both — never two
    orchestrators back-to-back; FRESH context each hop (no prompt-cache TTL bleed, no
    single context driving a 7-phase slice end-to-end)
```

**Trust tiers CHANGE from SL-199.** The orchestrator is no longer *confined*
(Mode-B: every coord write routed through server-side MCP); it is
**nominated-unjailed** (`PassThrough`, RW `.git`) and imports/commits with plain
git, exactly like the main-thread claude arm. Safety no longer rests on the
MCP-unreachability of coord `.git`; it rests on the **nomination + spawn-gate**
closed loop (§5.6 — proven P1/P3/P4). The **workflow** still holds no MCP and runs
no git — but it is no longer a "dumb sequencer": it is the **spawn authority and
per-phase state bus** (2a-a), threading `{ base_B, worker_fork, fork_tip }` from
each agent's schema into the next agent's prompt.

### 5.2 Interfaces & Contracts

**The emitter (new Rust, read-only).** Two surfaces over one reader:

- CLI: `doctrine dispatch phase-receipt --slice <N> --phase <PHASE-NN>` → typed JSON
- MCP: `dispatch_phase_receipt{slice, phase}` (granted to `dispatch-orchestrator`)

```rust
// The surface is truthful-by-construction: a coord refusal has no tip to report
// (F3), so refusal is a DISTINCT variant, not a core with a fabricated sentinel.
enum PhaseReceiptResult {
    Resolved(PhaseReceiptCore),
    CoordRefused(CoordRefusal),          // resolve_coord failed — no dispatch_tip exists
}
enum CoordRefusal { UnknownSlice, Ambiguous, Stale }

// durable, boundary-ledger-backed — the receipt's load-bearing truth
enum ReceiptStatus {
    NotStarted,          // no sheet progress, no boundary row
    InProgress,          // sheet in_progress, no boundary row
    Blocked,             // sheet=blocked — a control-flow boundary (state::PhaseStatus::Blocked, F4)
    Completed,           // boundary row exists for the phase on dispatch_tip (DURABLE)
    ConcludeIncomplete,  // sheet=completed ∧ boundary missing — retryable funnel fault (§5.4)
    Unknown,             // sheet malformed / unreadable — fail-loud, never silently "incomplete" (F4)
}

struct PhaseReceiptCore {
    slice: u32,
    phase: String,                       // PHASE-NN (immutable id)
    receipt_status: ReceiptStatus,       // durable, boundary-backed
    runtime_status: Option<SheetStatus>, // advisory, sheet-derived, nullable
    dispatch_tip: String,                // dispatch branch HEAD (NOT a code oid) — always present in Resolved
    boundary: Option<Boundary>,          // Some ⟺ boundary row exists
}
struct Boundary { code_start: String, code_end: String }  // code-range OIDs, distinct from dispatch_tip
```

`ReceiptStatus` is a **new** enum, NOT `state::PhaseStatus` (that is a sheet
lifecycle; reusing it would let callers infer durable guarantees from advisory
`runtime_status`). It **must, however, cover every non-completed state the
existing authorities distinguish** (F4): `Blocked` is a real control-flow
boundary in `compute_next_phases` (`src/dispatch.rs:3138`) and
`state::PhaseStatus::Blocked` (`src/state.rs`); a malformed/unreadable sheet maps
to `Unknown` (fail-loud), never a silent `NotStarted`. `dispatch_tip` and
`boundary.code_end` are **named by role** — both are commit OIDs but different
tips; never a generic `coord_tip`.

**Readiness (new Rust, read-only, slice-global).**
`dispatch_next_ready{slice}` → `{ next_ready: Vec<String> }` — a thin wrapper
over the **existing** `compute_next_phases` authority. Deliberately a separate
surface from the phase receipt (see D3): "what happened to phase X" and "what
should the slice do next" are different altitudes.

**The full phase receipt (harness-side `schema:`; orchestrator-composed).** The
orchestrator grafts its runtime facts onto the emitter core; the `schema:` on
the `agent()` call forces the shape and the harness validates it:

```
PhaseReceipt = PhaseReceiptCore                        // (from dispatch_phase_receipt Resolved; ABSENT on CoordRefused)
  + worker_fork   : string                             // ephemeral, reaped. pi arm: O `git worktree add`-ed it (knowable at prep). claude arm (B): HARNESS-minted at agent() spawn (agent-id-named) — UNKNOWABLE at prep; the disposing O DISCOVERS it from WorkReceipt.fork_tip, never from prep (C1)
  + verify        : { green: bool, failures: string[] }// orchestrator RAN the tests
  + halt_reason?  : string                             // set on any stop — incl. "coord:<reason>" when the emitter returned CoordRefused
  + fixup?        : { reason: string, instructions: string } // fixable worker delta → revive on fork (§5.4)
  + next_ready    : string[]                           // slice-global adjunct (from dispatch_next_ready) — labelled, not part of the durable core
```

Under the **two-job orchestrator** (§5.4, D10) this composition is the **dispose
half** of `HopReceipt`; `worker_fork` is threaded from the **prep half** of the
*prior* hop, and the **prep half** for the *next* worker rides alongside. §5.2
stays the emitter-core contract; §5.4 is the harness-schema shape the driver loop
consumes.

On a `CoordRefused` emitter result the orchestrator emits a **minimal** receipt
(`halt_reason="coord:<reason>"`, no core fields) — the driver halts on
`halt_reason`, so the refusal path needs no fabricated core (F3).

`worker_branch` + `verify` are orchestrator-supplied because coord state cannot
know them (the fork branch is reaped; verify comes from running tests).

### 5.3 Data, State & Ownership

- **Committed coord state** (dispatch branch): boundary rows — the receipt's
  durable truth. Read by the emitter.
- **Runtime sheet** (gitignored, coord-tree-local): advisory `runtime_status`
  only. The emitter never trusts it for `Completed`.
- **Ephemeral** (fork branch, reaped): `worker_branch` — never in coord state,
  always orchestrator-supplied.
- **The emitter reads the COORD tree only** via `resolve_coord(root, slice)`
  (resolves by slice-id server-side, NOT by caller cwd) → a driver launched from
  the primary edge tree still gets coord-truth, immune to the IMP-174 split-brain
  on the primary side. `resolve_coord` refusals (`unknown-slice|ambiguous|stale`)
  surface as `coord_error`, **first-class**, never collapsed into a phase
  anomaly.

### 5.4 Lifecycle, Operations & Dynamics

> **↻ CONFINED-PLACEMENT DELTA RETIRED (2026-07-06, unjail re-open).** The prior
> §5.4 spawned ONE **confined** orchestrator per phase (no `isolation`, jailed to
> coord-root, arm→fork→import→conclude in one agent); its load-bearing concern was
> **placement** — coord-root parking so `cwd_is_coord_root` fires for the nested
> worker fork. That whole model is superseded by §5.1. Under unjail: the
> orchestrator is **nominated-unjailed** (RW `.git` from any cwd — P1 side-probe:
> every `.git` is RW under PassThrough), realized as **alternating unjailed
> orchestrators** — each interior one disposing the previous worker's commit +
> prepping the next in ONE agent (never two back-to-back) — and it **no longer
> nests the worker** (wall #1 — the workflow is spawn authority). Coord-root parking is therefore **no longer
> load-bearing for git access**: O addresses the coord tree explicitly
> (`git -C .dispatch/SL-<slice>`) and still asserts it exists ∧ is on
> `dispatch/<slice>` before acting — now a *correctness* precondition
> (`halt_reason="coord:<reason>"` on miss), not a confinement one. The harness
> contract (F1: pure-literal `meta`, top-level body, `scriptPath` invoke) + slice
> guard (F2: parse `args`, validate positive integer, halt) carry forward unchanged
> below. The retired placement memory
> (`mem.pattern.dispatch.confined-orchestrator-placement-not-permission`) is scoped
> to the confined model — kept as history, not current.

**Driver loop (JS, `/drive-slice`):**

**Per-hop schemas (harness `schema:`, agent-composed).** Orchestrators and workers
alternate; the workflow threads facts (2a-a), never computes them:
- `WorkReceipt = { fork_tip: string|null, summary }` — the worker's committed fork
  tip (claude arm, (B) self-commit) or `null` (pi / fallback (A) — the disposing
  orchestrator reads the worktree diff).
- `HopReceipt` — the between-workers orchestrator's **combined** receipt, two halves
  plus a slice-global adjunct:
  - **dispose** (absent on the bootstrap O₀): §5.2 `PhaseReceiptCore + verify +
    halt_reason?` **plus `fixup?: { reason, instructions }`** — set when the worker
    delta is a *fixable* defect (verify-red-but-addressable, incomplete edit); absent
    ⇒ accepted-or-halted. Drives the bounded revive loop.
  - **prep** (absent on the terminal O and whenever no next phase is ready):
    `{ phase, arm: "claude"|"pi", base_B, worker_prompt, worker_fork }` — the next
    worker's context; on pi, O has already `git worktree add`-ed + spawned it. A
    **null prep is overloaded** (drive-complete vs prep-failed): on a *hard* prep
    failure (distill error, pi `git worktree add` failure) the O MUST set
    `halt_reason` (`coord:`/`funnel:`) rather than return a silent null; the driver
    additionally belts a silent omission via the `next_ready` cross-check (A1, §5.4).
  - `next_ready: string[]` — the slice-global adjunct (`dispatch_next_ready`);
    load-bearing for the A1 belt, not decoration.
  - **`fixup` and `prep` are MUTUALLY EXCLUSIVE** — a fixable defect revives the
    worker (no prep until dispose is accepted); an accepted dispose may prep. The
    driver loop *assumes* this exclusivity, so the `schema:` **enforces** it (`oneOf`:
    `fixup` set XOR `prep` set XOR neither-when-halted) — the harness rejects a
    receipt carrying both, rather than the loop silently taking the `fixup` branch
    and dropping a live prep.

```js
const SEED_PHASE_COST = 45_000;   // RFC-011-observed funnel ceremony (STD-001: rationale in comment)
const SOFT_CEILING    = 120_000;  // advisory dumb-zone; planning-only, NEVER gated
const MAX_FIXUP       = 2;        // bounded worker-fixup revivals per hop (§5.5); halt past it

let lastActual = null;
const report = { phases: [], halted: null, divergence: null };
// F2 slice guard: `args` may arrive as a JSON string (Workflow footgun). Parse,
// validate a positive integer, halt fail-closed — never let a probe guess a slice.
const slice = Number((typeof args === 'string' ? JSON.parse(args) : (args||{})).slice);
if (!Number.isInteger(slice) || slice < 1) throw new Error(`drive-slice: bad slice ${JSON.stringify(args)}`);
// The workflow holds NO MCP and runs NO git: every doctrine read/write is via a
// spawned agent. It is the SPAWN AUTHORITY + STATE BUS (2a-a) — it threads
// {base_B, worker_fork, fork_tip} between the alternating agents, never computes them.

// Bootstrap O₀: read next_ready + PREP the first worker. Prep-only — no previous
// worker to dispose, so on the CLAUDE arm it is READ-ONLY ⇒ the unprivileged
// dispatch-probe role (no nomination, one fewer privileged spawn/drive). Only the
// PI arm's O₀ needs unjail (it `git worktree add`s the first worker's tree).
const arm = driveArm(args);              // drive-level: 'claude' (v1) | 'pi'
const bootstrapType = arm === 'pi' ? 'dispatch-orchestrator' : 'dispatch-probe';
let hop = await agent(bootstrapPrompt(slice), { schema: HopReceipt }); // agentType: bootstrapType
if (!hop || hop.halt_reason) { return { ...report, halted:{ reason: hop?.halt_reason ?? HALT.NULL_RECEIPT } }; }

while (hop.prep) {                                        // prep present ⇒ a phase is ready to run
  const prep = hop.prep, phase = prep.phase;
  if (budget.total && budget.remaining() < (lastActual ?? SEED_PHASE_COST)) {
    report.halted = { reason: HALT.BUDGET_EXHAUSTED, phase }; break;
  }
  const before = budget.spent();

  // W_i (JAILED): claude arm = workflow-spawned worker, self-commits (B).
  //               pi arm = already spawned by the prior O; skip this agent().
  let work = prep.arm === 'claude'
    ? await agent(prep.worker_prompt, { schema: WorkReceipt, isolation: 'worktree' }) // agentType: dispatch-worker
    : { fork_tip: null };

  // O_i — ONE orchestrator, TWO jobs: DISPOSE W_i, then (iff accepted) PREP W_{i+1}
  // in the SAME agent. Bounded fixup loop first: a fixable defect revives W_i on its
  // fork and re-disposes; no prep happens until dispose is accepted.
  let fixups = 0;
  for (;;) {
    hop = await agent(hopPrompt(slice, phase, prep, work.fork_tip), { schema: HopReceipt }); // dispatch-orchestrator
    if (!hop) { hop = { halt_reason: HALT.NULL_RECEIPT, prep: null }; break; }
    if (!hop.fixup) break;                                // disposed: accepted (may carry prep) or halted
    if (++fixups > MAX_FIXUP) { hop = { halt_reason: HALT.FIXUP_EXHAUSTED, prep: null }; break; }
    // revive-on-fork (§5.5): fresh worker on the SAME fork (durable delta under (B))
    // + O's fixup notes. NOT SendMessage context-intact — fork-durable.
    work = await agent(fixupPrompt(prep, hop.fixup), { schema: WorkReceipt, isolation: 'worktree' });
  }

  lastActual = budget.spent() - before;                  // adaptive (whole hop, incl. worker + fixups)
  report.phases.push({ phase, ...hop });
  log(`phase ${phase}: ${lastActual/1000|0}k${fixups?` (${fixups} fixup)`:''}`);

  // Halt on the DISPOSE half — named, single-sourced (F6).
  if (hop.halt_reason)                     { report.halted={reason:hop.halt_reason, phase}; break; } // coord:/funnel:/FIXUP_/NULL_ (F3)
  if (hop.receipt_status === 'ConcludeIncomplete') { report.halted={reason:HALT.CONCLUDE_INCOMPLETE, phase}; break; }
  if (hop.receipt_status === 'Blocked')    { report.halted={reason:HALT.PHASE_BLOCKED, phase}; break; }  // (F4)
  if (hop.receipt_status !== 'Completed')  { report.halted={reason:`${HALT.ANOMALY}:${hop.receipt_status}`, phase}; break; } // Unknown (F4)
  if (!hop.verify.green)                   { report.halted={reason:HALT.VERIFY_RED, phase}; break; }
  // A1 — `hop.prep` is OVERLOADED: drive-complete vs prep-failed vs prep-skipped.
  // A clean dispose that then FAILS to prep (distill error, pi `git worktree add`
  // failure) also returns prep:null; treating null as done would report a truncated
  // drive as complete (violates F3 — success must not be an omission). Belt: a null
  // prep with a NON-EMPTY next_ready is an anomaly, not completion. (The disposing O
  // SHOULD also set halt_reason directly on a hard prep failure it detects — coord:/
  // funnel:; this cross-check catches a SILENT omission the receipt didn't name.)
  if (!hop.prep && hop.next_ready && hop.next_ready.length) {
    report.halted = { reason: HALT.PREP_INCOMPLETE, phase }; break;
  }
  // accepted ⇒ loop on hop.prep; null prep ∧ next_ready empty ⇒ drive genuinely done.
}
report.divergence = await divergenceProbe(slice);  // agent(dispatch-probe) → read-only divergence tool (§5.5)
return report;
```

**Halt-reason vocabulary (named contract, F6).** The loop branches on halt
reasons *as protocol* — so the vocabulary is a **closed, single-sourced set**,
not scattered literals. Two families:
- **Derived from Rust closed vocabs** (re-exported, not re-invented): `funnel:<reason>`
  from `FunnelOutcome::Refused.reason`; `coord:<reason>` from `CoordRefusal`. The
  design does not mint these strings — it forwards the authored enums.
- **Script-local** — a named `HALT` table in `/drive-slice` (the driver's only
  authored vocabulary): `{ NULL_RECEIPT, CONCLUDE_INCOMPLETE, PHASE_BLOCKED,
  ANOMALY, VERIFY_RED, BUDGET_EXHAUSTED, FIXUP_EXHAUSTED, PREP_INCOMPLETE }`. Single
  source; the loop references members, never inline literals (STD-001 in the JS
  reference). `FIXUP_EXHAUSTED` fires when a hop's bounded revive loop exceeds
  `MAX_FIXUP` without an accepted dispose — the worker could not fix its delta
  in-budget. `PREP_INCOMPLETE` (A1) fires when a hop disposed cleanly but returned a
  null `prep` while `next_ready` is non-empty — a prep that silently failed, not a
  genuine drive-complete (§5.4 loop belt).

- **null `total` → unmetered**: `budget.total &&` short-circuits; the loop runs
  all ready phases (D4-a).
- **adaptive cost**: `lastActual` = pool-spend delta across the one `agent()`
  call (D4-i).
- **`ConcludeIncomplete` contract**: the orchestrator's own remedy is to **retry
  conclude against the CURRENT live `dispatch_tip`**; a `Refused{lost-ref-race}`
  → the orchestrator sets `halt_reason="funnel:lost-ref-race"`. "Retryable" is
  NOT a guaranteed self-heal — a moved tip surfaces as a funnel race. If the
  orchestrator returns `ConcludeIncomplete` to the driver, the driver halts
  (no blind JS retry — the repair is the orchestrator's, on the live tip).
- **Funnel `Refused` → halt**: the orchestrator maps any funnel refusal to
  `halt_reason="funnel:<reason>"`, stops, returns a non-`Completed` receipt.
  Report-and-halt; never auto-merges.

**Grant boundary (the real safety gate).**

| Subagent type          | MCP grant (exhaustive)                                | Raw tools |
|------------------------|-------------------------------------------------------|-----------|
| `dispatch-orchestrator` **(nominated-unjailed)** | `dispatch_import`, `dispatch_conclude_phase`, `dispatch_reap`, `dispatch_phase_receipt`*, `dispatch_next_ready`*, `dispatch_authored_divergence`* | Read, Edit, Write, Bash, Grep, Glob — **no `Agent`** (stripped for workflow leaves, wall #1; O never nest-spawns — workflow is spawn authority; O uses Bash for the pi-arm `git worktree add` + subprocess) |
| **`dispatch-probe`** (new) | `dispatch_next_ready`*, `dispatch_authored_divergence`*, `dispatch_phase_receipt`* | **Read, Grep, Glob only** — no Write/Edit/Bash/Agent. Serves the closing divergence probe AND the **claude-arm bootstrap O₀** (prep-only, read-only ⇒ no nomination — least-privilege) |
| `dispatch-worker` (jailed) | `worker_commit` — `SubagentStart(dispatch-worker)` **fires for the workflow leaf** ⇒ DispatchRecord provisioned ⇒ resolves (open gate: P5, MCP-retention) | Read, Edit, Write, Bash, Grep, Glob |
| workflow script        | *(none — JS, no MCP)*                                 | *(none)* |

(* new, read-only MCP tools.)

**F5 — the read-only probes get a genuinely read-only role.** The bootstrap
planner + closing divergence probe do NOT reuse `dispatch-orchestrator`: that role
carries raw `Edit`/`Write`/`Bash`/`Agent` (`install/agents/claude/dispatch-orchestrator.md`),
so "read-only" would be a lie even though no new *MCP* write is added (CHR-039:
containment is by policy, not MCP-unreachability). A new **`dispatch-probe`** role
holds only the read-only MCP tools + `Read/Grep/Glob` — no raw write, no nested
spawn. The orchestrator retains its raw tools (it legitimately runs the funnel and
tests); only the standalone probe calls are narrowed.

**F1 — the conformance authority must grow in lockstep.** Granting the three new
MCP tools to `dispatch-orchestrator` (and the read set to `dispatch-probe`) is NOT
free: `allowed_mcp_tokens(role)` in `src/doctor_checks.rs` (doctor check #9,
**Error-severity** conformance on authored agent defs) currently allows the
orchestrator only the three funnel tokens. Adding tools without updating it
**false-reds** the very agent-def change this design depends on. ⇒ `src/doctor_checks.rs`
is a **declared touch-target** (§9 selectors); the role→allowlist authority grows
with the tools, and a test asserts the new set.

**The gate is checked two mechanically-real ways** (no runtime grant-readback API
exists — D2):
- **Static (source-conformance ONLY, F2)** — doctor check #9 + phase-0 inspect the
  **authored** allowlists (`install/agents/claude/dispatch-orchestrator.md`,
  `dispatch-worker.md`, and the new `dispatch-probe.md` — exact files in §9). This
  proves the *source* grant, NOT the *live* one: the harness runs the installed
  copy under `.claude/agents/`, and **ISS-216 (open) — `doctrine install` cannot
  reseat a changed agent def** — means the source scan is not proof of the runtime
  surface. So phase-0 carries an explicit **precondition**: a clean install/reseat
  of the changed defs (or a manual placement) BEFORE the grant story is trusted;
  the design claim is narrowed to "source-conformance + a verified live install",
  never "source scan ⇒ runtime truth".
- **Runtime** — phase-0 has the orchestrator attempt a forbidden `review_*`/`memory_*`
  write → confirmed absent/refused against the **installed** def; and confirms
  positive callability of the granted funnel tools.

### 5.5 Invariants, Assumptions & Edge Cases

- **Unjail placement, not coord-root parking (supersedes the PHASE-05 delta).** The
  orchestrator runs **nominated-unjailed** (`PassThrough`) — RW `.git` from any cwd
  (P1 side-probe). It is NOT jailed to coord-root; it addresses the coord tree
  explicitly (`git -C .dispatch/SL-<slice>`) and imports/commits with plain git (not
  the server-side MCP-only Mode-B path). Load-bearing instead: **(1) nomination must
  land before O's first tool call** (`SubagentStart` sync-blocking, §5.6/I2); **(2)**
  O still asserts the coord tree exists ∧ is on `dispatch/<slice>` before acting —
  now a *correctness* precondition (`halt_reason="coord:<reason>"` on miss), not a
  confinement one. The worker stays jailed either arm (bwrap / worktree-jail) —
  confinement is the worker's, never the orchestrator's.
- **Revive is fork-durable, not context-intact (§5.1, D10).** The documented
  Workflow primitives are `agent()`/`parallel()`/`pipeline()` — one-shot spawns, no
  `SendMessage`. So a fixup revives the worker as a **fresh `agent()`** whose delta
  starts from the prior worker's committed `fork_tip` (durable under (B)) carrying
  O's fixup notes — the worker resumes from committed state, not cold, but its prior
  *live reasoning* is gone. **The revive base is NOT free (C1).** A plain
  `agent(isolation:'worktree')` mints a **fresh** worktree off the Bash-cwd HEAD
  (armed base `B_i`), **not** the prior `fork_tip` — so "on the same fork" needs a
  stated mechanism, not an assertion. Two, in preference order:
  1. **Re-armed base (preferred, unproven).** The `WorktreeCreate` arming hook —
     doctrine's own, which already controls the isolated worker's fork base
     (CHR-039: payload is controllable) — points the revive worktree's base at
     `fork_tip` instead of `B_i`. Zero worker-side git. **Gated by OQ-6**: CHR-039
     proved the hook controls the fork *cwd*, not yet that it can redirect the fork
     *base ref* to an arbitrary existing branch tip.
  2. **Worker-side reset, base ENFORCED not hoped (floor).** The fixup prompt carries
     `fork_tip`; the revive worker resets its fresh worktree to it, re-edits, then
     **commits via `worker_commit` exactly as the initial worker does** — and that is
     where the guarantee actually lives, **not** in the reset. RV-258 F-5 is correct
     that a raw `git reset --hard` + prompt-obedience is **no guarantee** (R-5 belts
     only `.doctrine/`/`.claude/` writes, not ref movement; the repo carries a
     documented wrong-base trap — `mem.signpost.doctrine.dispatch-claude-arm-wrong-base`).
     The fix: the revive's `DispatchRecord` base is **re-stamped to `fork_tip`**, so
     `worker_commit` (`src/mcp_server/worker_commit.rs` — refuses unless `HEAD==base`,
     then verifies the new commit's single parent `== base`) **mechanically rejects a
     wrong-base revive** before any tip moves. Belt-and-brace on the **(A)** import
     path (no `worker_commit`): the **disposing O verifies the revive commit's parent
     chain descends from `fork_tip` before import**, halting `coord:revive-wrong-base`
     on a mismatch. So "same fork" is enforced by the same base-check that guards the
     initial commit, on both handoff modes — never by prompt obedience.
  Design ships mechanism 2 as the floor and adopts 1 iff OQ-6 proves out. The
  disposing O is likewise re-spawned each pass (re-derives from the fork diff +
  verify output). Accepted cost of workflow-spawned ephemerals; true context-intact
  revive is a later enhancement iff the runtime exposes continuation (OQ-5).
  `MAX_FIXUP`-bounded; exhaustion halts (`FIXUP_EXHAUSTED`).
- **Config lives in the script.** `SEED_PHASE_COST`, `SOFT_CEILING` are named
  constants in `/drive-slice` (harness-side, consumed by JS `budget.*`), NOT
  doctrine config — the Rust emitter never sees `budget.spent()`. No new Rust
  config surface (D4).
- **Advisory dumb-zone never binds at one-per-phase**: each orchestrator is a
  fresh context ≈ `lastActual`; `SOFT_CEILING` is a *batching hint* for the
  deferred follow-up (§8 R2), log-only.
- **IMP-174 — no auto-land.** The driver drives + reports; landing the fork's
  authored+code deltas to edge/main stays `/audit → /reconcile → /close` where
  IMP-174 is unresolved. Non-Goal (§ scope). The final report seeds a cheap raw
  advisory, **never gated, never acted on**:
  ```
  authored_divergence = { diverged: bool, compared_ref: string, drifted_paths?: string[] }
  ```
  computed `git diff --name-only <trunk_ref>..<dispatch_tip> -- .doctrine/**`,
  where `trunk_ref` is resolved from the **existing trunk authority** —
  `git::trunk_commit` (`src/git.rs`) or the `[dispatch] deliver_to` ref
  (`run_deliver_to`, `src/dispatch.rs`) — NOT the journal-row printer
  `run_show_journal_trunk_oid` (`src/dispatch.rs:630`; that prints a committed
  integrate-journal row, not a reusable trunk ref) and NOT hardcoded `edge`,
  recorded in `compared_ref`
  so the signal is not repo-local folklore. IMP-174 owns the partition /
  refuse-loud semantics. **Because the script has no shell**, this is a
  **read-only doctrine tool** (`dispatch_authored_divergence{slice}`, granted to
  the read-only `dispatch-probe` role — §5.4/F5), not raw JS `git` — it rides the
  same coord-resolution + trunk authority as the emitter. (The third read-only
  tool; no write surface — folded into the same grant-gate story as §5.4.)
- **`resolve_coord` refusals are workflow halt states**, not noise: `stale`
  (torn-down/prunable coord) and `ambiguous` are plausible from the primary tree
  and must halt loudly.

### 5.6 Security — nomination + spawn-gate (ADR-008 amendment)

The orchestrator runs **unjailed** (`PassThrough`, RW `.git`) — a new exception
to orchestrator confinement, so an **ADR-008 amendment is in scope** (§7, §9).

**Proof status is per-seam, not blanket (RV-258 F-1).** Escalation via the
**`Agent`** seam is **proven closed** end-to-end, no arming token — P1 (nomination
round-trip), P3 (discriminator), P4 (active deny), 2026-07-06
(`unjail-direction.md` §6). The **`Workflow`** seam (I1(b)) is **NOT yet
empirically proven** to carry caller identity into the gate (OQ-4). The design does
**not** wait on that proof to be safe: the Workflow seam is closed **by a
fail-safe default** — until OQ-4 proves a jailed caller's `agent()`/`Workflow` call
presents its `agent_id` to a `PreToolUse(Workflow)` matcher, the matcher **DENIES
`Workflow` to any caller with a present `agent_id`** (i.e. any subagent), full
stop. So the confinement boundary is closed *by construction* on both seams — one
by a proven identity-gate, one by a blanket subagent-deny — and OQ-4's outcome only
decides whether the Workflow seam can be *relaxed* to the finer identity-gate, never
whether it is safe. No seam is ever "assumed covered": a seam is gated or denied.

**Two halves, one closed loop.**

1. **Nominate.** A `SubagentStart(dispatch-orchestrator)` command hook writes the
   spawned `agent_id` to a PassThrough allowlist at a FIXED, out-of-jail path
   (`$CLAUDE_PROJECT_DIR`-resolved, never cwd-relative — the hook's cwd is the
   *spawned* tree, §4.3-1). The hook is **sync-blocking** — the entry lands before
   the orchestrator's first `PreToolUse` (P1b). `PreToolUse.decide()` then reads
   `agent_id ∈ allowlist → PassThrough`, scoped to that one id (a non-listed
   control stays jailed — P1a).
2. **Gate.** A `PreToolUse(Agent)` matcher **denies** the spawn iff
   `agent_id present ∧ caller ∉ allowlist ∧ subagent_type ∈ privileged-set`
   (`dispatch-orchestrator`, `fork`, …). `PreToolUse(Agent)` carries the caller's
   `agent_id` — present iff subagent-initiated (P3) — the parentage discriminator
   `SubagentStart` **lacks** (P0 Q1). So a jailed `Agent`-holder cannot spawn a
   privileged (auto-nominated) child: escalation closes at the source, and
   `SubagentStart` type-match is trustworthy again — the token that only
   compensated for the missing parentage is retired (P4).

**Design invariants.**
- **I1 — one list, EVERY spawn seam.** The gate's privileged deny-set IS the same
  authored list as the nomination-eligible-set — never two lists that can drift.
  A type added to nomination without the gate is an open escalation hole; the
  design binds them to one source so the gate cannot lag a new nomination. The
  invariant is **not** "one list, two consumers" — it is **one list, gating every
  spawn seam the harness exposes**. Today there are **two** spawn seams, not one:
  (a) the `Agent` tool (`PreToolUse(Agent)` — proven P3/P4), and (b) the **Workflow
  tool** — a jailed subagent holding `Workflow` (general-purpose agents hold `*`)
  can run a script whose `agent('…', {agentType:'dispatch-orchestrator'})` spawns a
  privileged child, **laundering the jailed caller's identity through the workflow
  runtime**. The same matcher MUST cover both: extend the `PreToolUse` gate to the
  `Workflow` tool — deny when `caller agent_id present ∧ caller ∉ allowlist` (a
  jailed caller cannot open a workflow that spawns privileged leaves). A *future*
  harness spawn surface inherits the gate by this rule, defaulting to **gated, not
  forgotten**. (This is what OQ-4 must resolve as a **symmetry requirement**, not a
  convenience: see §6.)
  **I1 is CHECKED, not asserted** (matches "containment by grant, checked" — §4). A
  new **doctor check** (a doctor-#9 sibling over the shipped hook config) fails
  **Error-severity** on either of two conditions: **(a)** any nominated
  `subagent_type` (from a `SubagentStart` matcher) is **not** in the gate deny-set
  (nomination ⊄ gate); **(b)** a **known spawn seam has no `PreToolUse` matcher at
  all** — the seam registry (`Agent`, `Workflow`) must each have a gate entry, else
  a nominated type could be "in the deny-set" while an *ungated seam* bypasses the
  deny check entirely (RV-258 F-1: nomination ⊆ gate is necessary, not sufficient —
  the gate must also provably cover every seam). Together (a)+(b) are the mechanical
  form of "one list, every spawn seam"; a new seam added to the registry without a
  matcher reds the check, defaulting to gated-not-forgotten.
- **I2 — config outside every jail.** The allowlist and hook config live at a
  path writable ONLY by the main thread — out of every worktree jail (jailed
  agents cannot write it: worktree-jail Bash deny + import R-5 belt on
  `.claude/`/`.doctrine/`). A worker therefore cannot nominate anything.

**Fail-safe — every failure lands on *more* confinement.** A missed
`SubagentStart` (not fail-closeable — exit 2 does not abort) leaves the
orchestrator **jailed**: a visible functional failure, never an escape. Absence
of an allowlist entry never grants `PassThrough`. Hygiene: a `SubagentStop`
matcher removes the entry so a stale `agent_id` cannot be reused (fires awaited,
tree-intact).

**Amendment ledger (for the ADR-008 amendment, D8).** The amendment records three
things beyond the grant: (a) the **confined orchestrator (Mode-B) as a reversible
escape hatch** — it loses nothing functional under Workflow (import already routes
via the shipped MCP), so if the seam-symmetry obligation below proves
unmaintainable, revert with no functional loss; (b) the **seam-symmetry obligation
as the standing price** — I1 binds *every* harness spawn seam (`Agent`, `Workflow`,
any future surface) to the one deny-set; a new seam defaults to gated, and that duty
is permanent for as long as the orchestrator runs unjailed. **This toll stays
one-time ONLY while it is conformance-checked** (the I1 doctor check): as prose it
would be re-paid per harness release. (c) the **steered-orchestrator blast radius,
named out loud.** The P1/P3/P4 proofs bound **jailed→O escalation** (a worker cannot
*become* the orchestrator); they say **nothing** about **O-steered** — the
orchestrator's primary attacker-shaped input is the worker's diff (fork→O), and a
nominated-unjailed O has main-thread reach (primary tree, history, `.claude/`), a
reach Mode-B's tunnel-set bounded. The residual is roughly what the main thread
already accepts running `/dispatch` by hand, and is mitigated (trusted committed
def, narrow prompt, **structured inputs — diffs + typed receipts, not free-form
instructions**) — but the amendment states the residual as a sentence rather than
letting "proven safe P1/P3/P4" imply the escalation proof covers it. It does not.

**Consistent with ADR-006 / CHR-039.** The orchestrator is unconfined *by design*
(sole writer). Nomination relocates the main thread's existing unjailed status to
a trusted-def subagent — the def name cannot be forged (`.claude/agents/` is
worker-unwritable: jailed + R-5) — it invents no new trust. Note the shipped
`dispatch-worker` provisioning matcher is escalatable *today* (bounded — worker
commits land on an ephemeral fork behind the commit gate); the same spawn-gate
hardens it. Rule: **grant power must scale with intent evidence** — type-match
alone buys bounded/gated capability (worker tier); anything unbounded
(`PassThrough`) requires the spawn-gate.

## 6. Open Questions & Unknowns

- **OQ-1 — RESOLVED to a baseline (RV-258 F-4).** `/drive-slice`'s authored home is
  **`install/workflows/drive-slice.js`** (committed, audited), seeded to
  `.claude/workflows/drive-slice.js` (the installed/derived copy) by `doctrine
  install`. This is now a **design decision, not an open question** — the script is a
  primary deliverable (scope) and must not sit outside the audit surface; leaving the
  home unpinned let the *currently shipped* `install/workflows/drive-slice.js` (the
  retired confined-per-phase POC) drift unflagged. ⇒ `install/workflows/drive-slice.js`
  is a **design-target selector** (§9), so conformance reds the stale POC script
  until the implementation phase replaces it with the alternating-unjailed topology.
  Phase-planning confirms only the *install mechanics* (the seeding leg in
  `src/install.rs`), not the home. (The `.claude/workflows/` copy stays gitignored
  scope-relevant — the installed artifact, never the authored source.)
- **OQ-2** — Does `compute_next_phases` need a machine-readable (JSON) surface,
  or does `dispatch_next_ready` wrap it cleanly without touching the existing
  CLI rendering? (Expected: thin wrapper, no CLI change.)
- **OQ-3 — (B) worker self-commit, one residual probe (P5).**
  `SubagentStart(dispatch-worker)` **fires for a workflow-spawned worker leaf**
  (operator-confirmed 2026-07-06) ⇒ the `DispatchRecord` is provisioned ⇒
  `worker_commit` resolves. The one open gate on (B) is **P5**: does a workflow
  leaf **retain the `worker_commit` MCP tool** (the runtime strips
  `Agent`/`Grep`/`Glob` and injects `StructuredOutput`; MCP retention unprobed,
  operator-assessed low-risk)? If P5 fails, fall to **(A)**: O(dispose) imports the
  worker's uncommitted working-tree diff. Design authors (B) target + (A) fallback
  (D9).
- **OQ-4 — spawn-seam SYMMETRY (not "either passes").** The workflow spawns
  `dispatch-orchestrator` (privileged) via `agent()` — a **second spawn seam**
  distinct from the `Agent` tool the §5.6 gate proved (P3/P4). Framing this as
  "confirm either the workflow presents `agent_id=<NONE>` OR doesn't trip
  `PreToolUse(Agent)`" is **backwards**: the convenient answer (workflow `agent()`
  reads main-thread-equivalent, or fires no gate) is **precisely the hole** — a
  jailed `Workflow`-holder then launders its identity through the runtime to spawn
  a privileged leaf (I1(b)). The requirement is a **symmetry invariant**: *every*
  spawn seam must present the caller's `agent_id` to the gate, or be denied to
  jailed callers outright. Concretely — phase-0 must establish, for the `Workflow`
  seam: (1) does a jailed subagent's `agent()`/`Workflow` call carry the caller's
  `agent_id` to a `PreToolUse(Workflow)` matcher? If yes, the I1 deny-set gates it
  for free. If no (the runtime hides caller identity), then `Workflow` MUST be
  **denied to jailed callers wholesale** (matcher on `Workflow` + `caller agent_id
  present`). The legit main-thread `/drive-slice` launch (`agent_id=<NONE>`) passes
  either way. Nomination *firing* for a workflow leaf is already confirmed (OQ-3);
  this is the **escalation** direction, unproven, and load-bearing for §5.6.
- **OQ-5 — revive-on-fork vs context-intact.** v1 revive is a fresh worker
  `agent()` on the fork (durable delta + O's notes) — documented primitives only.
  If the Workflow runtime later exposes `SendMessage`-style continuation,
  context-intact revive is a token-saving enhancement (RFC-011). Not v1.
- **OQ-6 — revive-base mechanism (C1).** Can the `WorktreeCreate` arming hook
  redirect a revive worktree's fork *base ref* to an existing `fork_tip` (mechanism
  1, §5.5), or only its cwd (CHR-039 proved cwd, not base-ref)? Phase-0 probe. If it
  cannot, the design falls to mechanism 2 (worker-side `git reset --hard <fork_tip>`,
  guaranteed) — no architecture change either way, only which revive path ships.

## 7. Decisions, Rationale & Alternatives

- **D1 — Rust-primary, canonical read-only emitter.** Alternatives: script-primary
  (control flow stays prose-parse; nothing to TDD) rejected; both-first-class
  (widest blast radius) deferred. The receipt schema is the durable, testable,
  reusable asset.
- **D2 — Grant gate = static allowlist + runtime negative-probe**, not a runtime
  grant-readback (no such API). The safety claim lives in a tested artifact (the
  authored allowlists, held by conformance), not policy prose.
- **D3 — `next_ready` is a labelled slice-global adjunct via a separate
  `dispatch_next_ready` surface**, NOT baked into the durable `PhaseReceiptCore`.
  Keeps altitudes clean (phase-local truth vs slice-global readiness); avoids a
  per-phase receipt implicitly re-reading all phases as part of its *core*
  contract.
- **D4 — null-total unmetered (a) + adaptive cost (i); config in-script.** The
  budget is a ceiling only when set; adaptive cost avoids a rotting magic number.
- **D5 — Status truth from the committed boundary row, `ReceiptStatus` a new
  durable enum with an explicit `ConcludeIncomplete`.** The funnel's documented
  half-landed self-healing state must be a first-class receipt state, not generic
  incompletion — else the driver misreads a retryable funnel fault.
- **D6 — Narrow `phase_projection` extract.** `run_status` delegates only its
  per-phase row construction to a new read-only projection over {plan, sheet,
  committed boundaries}; all slice-global aggregate logic stays in `run_status`
  (behaviour-preserving). A wholesale reader merge would bleed slice-global policy
  into the emitter — rejected.
- **D7 — RV-255 inquisition integrations.** (F1) the role→MCP allowlist authority
  `allowed_mcp_tokens` (`src/doctor_checks.rs`, check #9) grows in lockstep with
  the new tools — a declared touch-target, not an afterthought. (F2) the static
  grant claim is **source-conformance + a verified live install**, never "source
  scan ⇒ runtime truth" — ISS-216 (install-reseat gap) makes phase-0's clean-
  install precondition load-bearing. (F3) the emitter surface is
  `PhaseReceiptResult = Resolved(core) | CoordRefused` — truthful by construction,
  no fabricated tip on refusal. (F4) `ReceiptStatus` covers `Blocked` + `Unknown`
  — every non-completed state the existing authorities distinguish, fail-loud on
  malformed. (F5) standalone read-only probes get a genuinely read-only
  `dispatch-probe` role, not the raw-tool-bearing orchestrator. (F6) the halt
  vocabulary is a named contract (`HALT` + re-exported `funnel:`/`coord:` closed
  vocabs). (F7) scope prose corrected to three read-only tools.
- **D8 — orchestrator nominated-unjailed; safety by nomination + spawn-gate, not
  MCP-unreachability.** Supersedes SL-199 Mode B (confined orchestrator, every
  coord write via server-side MCP). Chosen for **simplicity of reasoning + lower
  implementation complexity** (operator's real kicker), not necessity — a confined
  orchestrator was proven viable (P0 Q2) but keeps the whole server-side-MCP-only
  coord-write constraint. Cost, stated honestly for the ADR-008 amendment ledger:
  unjail buys plain-git convenience for O but adds a **standing obligation** —
  nomination machinery + the spawn-gate + `SubagentStop` hygiene + the amendment +
  the **seam-symmetry duty forever** (I1: every future harness spawn surface inherits
  gate duty, B1/OQ-4). The confined alternative loses **nothing functional** under
  Workflow (import already routes via the shipped MCP). So the amendment MUST record
  two things: (a) the **confined orchestrator as a reversible escape hatch** — if the
  seam-symmetry obligation ever proves unmaintainable, revert to Mode-B with no
  functional loss; (b) the seam-symmetry obligation itself as the **ongoing price**
  of the unjail posture. This strengthens the amendment; it does not change the
  verdict. Proven safe P1/P3/P4 (§5.6); no arming token.
- **D9 — spawn/import authority split; (B) self-commit target, (A) import
  fallback.** The workflow spawns all agents (wall #1: an orchestrator leaf has no
  `Agent` tool → cannot nest the worker); the disposing orchestrator imports +
  commits — a forced departure from `/dispatch` (there the orchestrator both spawns
  and imports), but it buys fresh-context-per-hop (no TTL bleed). Claude worker
  **self-commits via `worker_commit`** (fork-durable ⇒ O reads a committed tip;
  revive resumes from it). **(A)** orchestrator-imports-diff is the both-arm
  in-a-pinch fallback (pi arm is always (A) — worker bwrap-confined, cannot commit;
  `pi-spawn-confined.sh`). Rejected authoring (A) primary: ephemeral-worktree
  survival fragility + no durable revive point. Pi arm kept reachable, not a v1
  priority.
- **D10 — one orchestrator between two workers (two jobs), bounded revive-on-fork.**
  No two orchestrators back-to-back: each interior O disposes the previous worker's
  commit *and* preps the next in ONE agent (`HopReceipt`); boundaries degenerate
  (O₀ preps-only, terminal O disposes-only). Fixup = fresh worker on the fork + O's
  notes, `MAX_FIXUP`-bounded, **not** `SendMessage` context-intact (unavailable
  in-workflow, OQ-5); O re-spawned each dispose pass (re-derives from the fork diff
  + verify). Accepted cost of workflow-spawned ephemerals for fresh-context
  isolation.

## 8. Risks & Mitigations

- **R1 — IMP-174 split-brain at the drive→close handoff.** The driver inherits
  the unresolved authored-vs-edge divergence. *Mitigation*: no auto-land; read
  coord-committed truth only; seed the raw divergence advisory; `SL-206 related
  IMP-174` records the seam. Residual: full reconciliation is IMP-174's, not
  this slice's.
- **R2 — Batching-per-orchestrator not built** → `SOFT_CEILING` is inert control
  flow today. *Mitigation*: shipped as advisory-only + a named follow-up; no
  pretence it gates.
- **R3 — Un-allowlisted MCP *writes* run un-prompted from a background worker**
  (CHR-039 tested reads only). *Mitigation*: the grant gate (D2) is the control;
  the emitter adds no write surface; phase-0 runtime-probes a forbidden write.
- **R4 — worker fork-at-base from a workflow leaf not empirically re-demoed**
  (unjail re-frame: the fork is now the **worker's**, minted by the workflow
  `agent(isolation:'worktree')` leaf, NOT a nested confined-orchestrator spawn —
  CHR-039 proved the leaf fires `WorktreeCreate` and forks at the armed base).
  *Mitigation*: plan **phase-0** is a narrow de-risk (one real `/drive-slice`
  against a scratch slice, asserting the worker fork mints at armed base on
  `dispatch/<n>` end-to-end, and — folded in — the OQ-6 revive-base probe); does NOT
  re-run CHR-039's settled probes.
- **R5 — behaviour-preservation regression** in `run_status`. *Mitigation*: the
  extract is delegate-only; existing dispatch suites must stay green unchanged
  (the gate).
- **R6 — ISS-216 install-reseat gap (RV-255 F2).** The changed agent defs
  (`dispatch-orchestrator`, new `dispatch-probe`) may not reseat into
  `.claude/agents/` via `doctrine install`, so the live grant surface can lag the
  authored one. *Mitigation*: phase-0's clean-install precondition + a runtime
  probe against the **installed** def; do not trust the source scan alone.
  Residual: ISS-216 itself is out of scope (its own issue) — SL-206 depends on a
  manual reseat until it lands.

## 9. Quality Engineering & Validation

**Unit (Rust, TDD red/green/refactor):**
- `phase_projection` — per-phase status derivation over every branch: boundary-present ⇒
  `Completed`; sheet-completed ∧ boundary-missing ⇒ `ConcludeIncomplete`;
  sheet-blocked ⇒ `Blocked`; malformed/unreadable ⇒ `Unknown` (fail-loud);
  in_progress/none ⇒ `InProgress`/`NotStarted`. Table-driven over fixture coord
  trees (ride existing dispatch test rig).
- `dispatch_phase_receipt` — returns `CoordRefused(<reason>)` for each
  `resolve_coord` refusal (no fabricated tip — F3); `Resolved` carries a real
  `dispatch_tip` distinct from `boundary.code_end`.
- `dispatch_next_ready` — agrees with `compute_next_phases` on the same fixtures.
- `dispatch_authored_divergence` — `diverged` true iff `.doctrine/**` differs
  `trunk_ref..dispatch_tip`; `compared_ref` = the resolved trunk authority (not
  hardcoded `edge`); read-only, no coord mutation.
- **`allowed_mcp_tokens` (doctor check #9, F1)** — asserts the orchestrator set
  grows to include the three read-only tools and the new `dispatch-probe` role
  holds exactly the read set; the check stays green on the updated agent defs.
- **I1 spawn-seam-symmetry doctor check (new, #9 sibling)** — over the shipped hook
  config, two Error conditions (RV-258 F-1): (a) every `SubagentStart` nomination
  `subagent_type` ∈ the `PreToolUse` gate deny-set (nomination ⊆ gate); (b) every
  seam in the seam registry (`Agent`, `Workflow`) has a `PreToolUse` matcher. Tests:
  a nomination with no gate entry reds (a); a registry seam with no matcher reds (b);
  the shipped config passes both.
- **Wrong-base revive rejection (RV-258 F-5)** — a revive whose delta does not
  descend from the re-stamped `fork_tip` base is **rejected before any tip moves**:
  under (B) by `worker_commit`'s existing `HEAD==base` ∧ single-parent-`==base`
  check (add the revive-base case to its test); under (A) by the disposing O's
  parent-chain verify (`coord:revive-wrong-base` halt). No test relies on the worker
  obeying the reset prompt.
- **Behaviour-preservation**: existing `dispatch status` suite green **unchanged**
  after the `phase_projection` extract.

**Static / conformance:**
- Design-target agent-def files (recorded as selectors):
  `install/agents/claude/dispatch-orchestrator.md` (gains the three read tokens)
  and the new `install/agents/claude/dispatch-probe.md` — asserted (via doctor
  check #9) to grant exactly the intended tool sets. `dispatch-worker.md` is a
  **scope-relevant** selector, NOT a design-target: no phase modifies it; it is a
  behaviour-preservation assertion (doctor #9 stays green unchanged), and a
  design-target with no source delta invites a false under-delivery flag at audit.
  This proves the **authored source** grant only. The **live** grant under
  `.claude/agents/` requires a clean install/reseat, which ISS-216 (open) does not
  currently guarantee (F2) — so phase-0 carries an explicit clean-install
  precondition + a runtime probe against the installed copy, and the claim is
  never "source scan ⇒ runtime truth".

**Declared touch-set (selector accuracy, post-RV-255 + RV-258 F-4).** The Rust
design-targets are `src/dispatch.rs` (phase_projection + ReceiptStatus),
`src/mcp_server/dispatch.rs` (the three emitter tools), `src/mcp_server/tools.rs`
(MCP registration), and `src/doctor_checks.rs` (check #9 allowlist growth **+ the
new I1 seam-symmetry check**). The `/drive-slice` authored home **`install/workflows/
drive-slice.js` IS a design-target** (OQ-1 resolved) — registered `--intent
design-target`, so conformance reds the currently-shipped **retired-model** POC
script until the implementation phase overwrites it with the alternating-unjailed
topology (this is the drift F-4 flagged, now caught by the selector rather than
missed). `.claude/workflows/**` stays scope-relevant AND gitignored (the installed
copy only). `src/install.rs` (the probe asset + `install/workflows/` seeding leg)
rides the broad `src/**` scope-relevant selector — touched but not audit-red.

**Phase-0 e2e (manual, doubles as the fork-at-base demo + the `/drive-slice`
inspection verification):** one real `/drive-slice` against a scratch slice —
- the **worker** fork mints at armed base on `dispatch/<n>` (unjail re-frame — the
  fork is the workflow leaf's, not a nested confined-orchestrator spawn);
- a phase drives to `Completed`; an injected red verify halts the loop without
  auto-merge; the forbidden-write runtime probe refuses;
- **P5 pinned directly (RV-258 F-2), NOT assumed.** The claude arm must *prove which
  handoff mode ran*: either **(B)** — a **successful `worker_commit`** call landing a
  **non-null `WorkReceipt.fork_tip`** (the target path exercised end-to-end) — **or**
  an **explicit, recorded fall-to-(A)** with the P5 failure reason (worker leaf lost
  the `worker_commit` MCP tool). A silent degrade to (A) is a phase-0 **failure**:
  the drive must never *assume* (B) without evidence it ran;
- **Workflow-seam gate (RV-258 F-1).** Assert the OQ-4 posture live: a jailed
  subagent attempting a privileged `agent()`/`Workflow` spawn is **denied** (either
  by caller-identity if the seam carries it, or by the blanket subagent-deny default
  §5.6) — the escalation attempt never spawns a privileged leaf.

## 10. Review Notes

Internal adversarial pass folded in (see §7 D2/D3/D5/D6 — each is a review
finding integrated). External architect-review panel (codex / GPT-5.5) run
pre-write across §1/§1a/§2, three rounds; all findings accepted and integrated
(reader-seam narrowed to a delegate-only `phase_projection`; `ReceiptStatus`
gains `ConcludeIncomplete`; `coord_tip`→`dispatch_tip` role-named;
`resolve_coord` refusals first-class; divergence ref de-hardcoded to the trunk
authority; grant gate reframed to static+runtime probes; `next_ready` split to a
separate slice-global surface). Panel reported no residual stop-ship blocker.

**RV-255 (formal inquisition, GPT-5.5 on the written artifact).** 7 findings
(5 major, 2 minor, 0 blocker), all confirmed against real code and integrated
(§7 D7): F1 doctor-check-#9 allowlist authority now a touch-target; F2 grant claim
narrowed to source-conformance + verified install (ISS-216 dependency); F3
`PhaseReceiptResult` refusal variant; F4 `Blocked`/`Unknown` states; F5 read-only
`dispatch-probe` role; F6 named halt vocabulary; F7 scope prose corrected. No
blocker — the approach held; the gaps were artifact completeness/accuracy. Verdict
+ penance sealed in RV-255 `## Synthesis`.

**Post-seal accuracy corrections (plan-review pass, no architecture change).** Two
factual fixes folded back from the SL-206 plan review: (1) §5.5 trunk authority
re-pointed from the mis-cited `run_show_journal_trunk_oid` (`dispatch.rs:630`,
a journal-row printer) to the real resolver `git::trunk_commit` / `[dispatch]
deliver_to`; (2) §9 selector accuracy — `dispatch-worker.md` reclassified
design-target→scope-relevant, the Rust design-targets enumerated, and the
`/drive-slice` home flagged as an unregistered selector pending OQ-1. Both are
completeness/accuracy corrections, not architecture — the sealed verdict stands.

**Internal adversarial pass on the §5 unjail rewrite (2026-07-06, post-topology
correction).** Four findings, all integrated, no architecture change:
- **B1 (security, real)** — the spawn-gate covered one seam (`Agent`); the
  **Workflow tool is a second, ungated seam** (a jailed `Workflow`-holder can launder
  identity through the runtime). Fixed: I1 generalized to *one list, every spawn
  seam*; OQ-4 reframed from "either passes" to a **symmetry requirement** (§5.6 I1,
  §6 OQ-4).
- **A1 (correctness, real)** — `hop.prep === null` was overloaded (drive-complete vs
  prep-failed); a clean dispose that failed to prep read as done. Fixed: null-prep +
  non-empty `next_ready` ⇒ `HALT.PREP_INCOMPLETE` belt + hard-failure sets
  `halt_reason` (§5.4).
- **C1 (under-specified)** — revive-on-fork asserted "same fork" with no mechanism;
  a plain isolated `agent()` forks off `B_i`, not `fork_tip`. Fixed: two stated
  mechanisms (re-armed base, preferred/OQ-6; worker-side `git reset --hard`,
  guaranteed floor) + `worker_fork` provenance corrected (claude arm: discovered
  from `WorkReceipt`, not prep) (§5.2, §5.5, §6 OQ-6).
- **D8-ledger (sharpening)** — the ADR-008 amendment must record the confined
  orchestrator as a **reversible escape hatch** (no functional loss) and the
  **seam-symmetry obligation as the standing price** (§5.6 amendment ledger, §7 D8).
  Verdict unchanged.

**RV-258 (formal inquisition, GPT-5.5 on the written artifact, 2026-07-06).** 5
findings (1 blocker, 4 major), all confirmed against real code and integrated; no
architecture change — the corrections closed overclaims and hardened invariants:
- **F-1 (blocker) — safety overclaimed per-seam.** §5.6 read "provably safe /
  escalation closed" while the `Workflow` seam (I1(b)) is unproven (OQ-4). Fixed:
  proof status is now **per-seam** — the `Agent` seam is proven closed; the
  `Workflow` seam is closed by a **fail-safe blanket subagent-deny default** until
  OQ-4 relaxes it to the identity-gate. The I1 doctor check now also asserts **every
  seam has a matcher** (nomination ⊆ gate is necessary, not sufficient) (§5.6, §9).
- **F-2 (major) — (B) target unverified.** §9 never pinned P5. Fixed: phase-0 must
  prove which handoff mode ran — a successful `worker_commit` + non-null `fork_tip`,
  **or** an explicit recorded fall-to-(A); a silent degrade is a phase-0 failure (§9).
- **F-5 (major) — "guaranteed" revive floor was prompt-obedience.** Worker-side
  `git reset --hard` bypasses `worker_commit`'s proven base invariants. Fixed: the
  revive commits via `worker_commit` with base **re-stamped to `fork_tip`** (rejects
  wrong-base mechanically); the (A) path adds a disposing-O parent-chain verify
  (`coord:revive-wrong-base`); §9 pins a wrong-base-revive rejection test (§5.5, §9).
- **F-4 (major) — central artifact outside the audit surface.** OQ-1 left
  `/drive-slice`'s home unpinned while the shipped POC script still carried the
  retired model. Fixed: OQ-1 **resolved** — home is `install/workflows/drive-slice.js`,
  a **design-target** so conformance reds the stale POC until implementation replaces
  it (§6 OQ-1, §9).
- **F-3 (major)** — shell-damaged duplicate of F-5; **withdrawn** (raised in error).
Verdict: the approach held; every charge was artifact overclaim/completeness, not a
structural defect. No unresolved blocker. Sealed in RV-258 `## Synthesis`.
