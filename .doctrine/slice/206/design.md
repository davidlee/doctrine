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

The workflow is the **durable serial loop and the sole spawn authority**; each
phase runs through three *ephemeral* agents it spawns in turn — two
**nominated-unjailed** orchestrators bracketing one **jailed worker**. Spawn
authority (workflow) is deliberately **separated from import authority**
(`O(dispose)`) — a genuine departure from shipped `/dispatch`, forced by wall #1:
a workflow leaf has no `Agent` tool, so an orchestrator leaf cannot spawn the
worker; only the workflow can.

```
Workflow /drive-slice   (JS; args:{slice}; durable serial loop; SPAWN AUTHORITY)
  bootstrap: planner agent() → next_ready[]            (compute_next_phases authority)
  loop while next_ready non-empty & budget allows — per phase, ALTERNATING ephemeral agents:

    O(prepare) = agent(dispatch-orchestrator, schema: PrepReceipt)   ── NOMINATED-UNJAILED
       · SubagentStart(dispatch-orchestrator) → allowlist → PreToolUse PassThrough (§5.6)
       · capture base B = dispatch_tip; distill self-contained worker prompt
       · pi arm ONLY: `git worktree add` + spawn pi subprocess (RW .git — unjailed O)
       ▶ returns { base_B, worker_prompt, worker_fork }         → workflow holds (2a-a)

    W(work)    = claude arm: agent(dispatch-worker, isolation:worktree, schema: WorkReceipt)
                 pi arm:     (already spawned by O(prepare))    ── JAILED (bwrap / worktree-jail)
       · edits source in its fork; self-commits via worker_commit MCP  [claude arm, TARGET 2b-B]
       ▶ returns { fork_tip, summary }                          → workflow holds

    O(dispose) = agent(dispatch-orchestrator, schema: PhaseReceipt) ── NOMINATED-UNJAILED
       · import worker delta onto B; run verify (tests); commit ONE on dispatch branch
       · dispatch_conclude_phase → boundary row; dispatch_reap; dispatch_phase_receipt
       · on fixable defect: emit fixup verdict → workflow revives worker on fork (§5.4)
       ▶ returns PhaseReceipt (schema-validated: receipt_status | halt_reason | fixup)

    JS: receipt.fixup → revive W on fork; halt_reason/receipt_status → advance | halt+report
  → advance to next phase: FRESH ephemeral agents (no prompt-cache TTL bleed;
    no single context driving a 7-phase slice end-to-end)
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
  + worker_branch : string                             // orchestrator armed/spawned it (ephemeral, reaped)
  + verify        : { green: bool, failures: string[] }// orchestrator RAN the tests
  + halt_reason?  : string                             // set on any stop — incl. "coord:<reason>" when the emitter returned CoordRefused
  + next_ready    : string[]                           // slice-global adjunct (from dispatch_next_ready) — labelled, not part of the durable core
```

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

> **PHASE-05 DELTA (2026-07-06 — placement correction, evidence-backed).** The
> original §5.4 spawned the per-phase orchestrator with `isolation:'worktree'`.
> That is WRONG: `isolation:'worktree'` FRESH-FORKS the orchestrator into a
> detached `.worktrees/agent-<hex>`, where the SL-199 coord-root arming
> discriminator (`cwd_is_coord_root ∧ coord_in_dispatch`) never fires — so it
> cannot arm/fork/import and no phase completes. The confined-orchestrator MODEL
> is correct and UNCHANGED: the RO shared `.git` is BY DESIGN (SL-199 Mode B,
> proven — the orchestrator writes coord `.git` only via the server-side MCP funnel
> tools; "integrity never rests on the confined orchestrator"). The defect was
> **placement, not permission.** Corrections:
>
> 1. **Orchestrator spawn drops `isolation`.** A no-isolation subagent starts in
>    the driver's current working directory (`docs/claude/subagents.md:263`), so a
>    driver parked at coord-root places the orchestrator jailed to the COORD TREE
>    (`dispatch/<n>`, RO `.git`, writable working tree) — exactly SL-199's placement.
> 2. **Coord-root precondition (fail-closed).** `/drive-slice` MUST be invoked with
>    cwd parked at the coord tree (`.dispatch/SL-<n>` on `dispatch/<n>`) — the same
>    "cd into the coord tree" ritual `/dispatch` documents. Because the JS driver has
>    no shell, the check lives in the orchestrator's FIRST action: it asserts
>    `pwd == coord-root ∧ branch == dispatch/<slice>` and, on mismatch, returns a
>    minimal receipt with `halt_reason="coord:not-at-coord-root"` (§5.5). The driver
>    halts the drive on it like any `coord:<reason>`.
> 3. **Worker isolation stays def-pinned** to the worker frontmatter (SL-199 delta
>    A2), NOT the orchestrator's per-call arg — the nested worker forks at B off the
>    orchestrator's coord-root cwd, and `cwd_is_coord_root` fires for it.
> 4. **Harness contract (F1) + slice guard (F2).** `meta` is a pure literal and all
>    drive logic runs in a TOP-LEVEL body (the Workflow tool executes the top level,
>    never a `run` export; `.claude/workflows/` is not a named registry — invoke by
>    `scriptPath`). The driver parses `args` (which may arrive as a JSON string),
>    validates `slice` is a positive integer, and halts fail-closed otherwise; probe
>    agents never guess a slice.
>
> Evidence (do not re-derive): PHASE-05 acceptance drive (notes.md FINDING 3,
> corrected) + a placement spike — arm A (no isolation, driver parked at coord-209)
> landed at `.dispatch/SL-209` on `dispatch/209`, RO `.git`, writable working tree
> (= SL-199 placement); arm B (`isolation:'worktree'`) fresh-forked to a detached
> `.worktrees/agent-<hex>`. Corroborated by `docs/claude/subagents.md:263` +
> `settings.md:339` (baseRef default `head`). Durable:
> `mem.pattern.dispatch.confined-orchestrator-placement-not-permission`.

**Driver loop (JS, `/drive-slice`):**

```js
const SEED_PHASE_COST = 45_000;   // RFC-011-observed funnel ceremony (STD-001: rationale in comment)
const SOFT_CEILING    = 120_000;  // advisory dumb-zone; planning-only, NEVER gated

let lastActual = null;
const report = { phases: [], halted: null, divergence: null };
// F2 slice guard: `args` may arrive as a JSON string (Workflow footgun). Parse,
// validate a positive integer, halt fail-closed — never let a probe guess a slice.
const slice = Number((typeof args === 'string' ? JSON.parse(args) : (args||{})).slice);
if (!Number.isInteger(slice) || slice < 1) throw new Error(`drive-slice: bad slice ${JSON.stringify(args)}`);
// NB: the script has no shell / no MCP — every doctrine read is via a spawned
// agent. `planner`/`divergenceProbe` below are lightweight agent() calls granted
// only the relevant read-only tool, NOT raw JS.
let ready = await planner(slice);   // agent() → dispatch_next_ready{slice} (compute_next_phases authority)

while (ready.length) {
  const phase = ready[0];
  const est = lastActual ?? SEED_PHASE_COST;
  if (budget.total && budget.remaining() < est) { report.halted = {reason:'budget-exhausted', phase}; break; }

  const before = budget.spent();
  // PLACEMENT: NO isolation. A no-isolation subagent starts in the driver's cwd
  // (subagents.md:263); with the driver parked at coord-root the orchestrator is
  // jailed to the COORD TREE (dispatch/<n>, RO .git, writable working tree) — the
  // proven SL-199 placement where cwd_is_coord_root fires for the nested worker
  // fork. isolation:'worktree' would fresh-fork it to a detached .worktrees/agent-<hex>
  // where arming can never fire. Worker isolation rides the worker frontmatter, not here.
  const r = await agent(orchestratorPrompt(slice, phase),
                        { schema: PhaseReceipt });   // agentType: dispatch-orchestrator
  lastActual = budget.spent() - before;               // adaptive
  report.phases.push(r);
  log(`phase ${phase}: ${lastActual/1000|0}k; soft-ceiling headroom ~${((SOFT_CEILING-lastActual)/1000)|0}k`); // advisory

  // Halt vocabulary is a NAMED, single-sourced contract — see HALT in §5.4 (F6).
  if (!r)                                { report.halted={reason:HALT.NULL_RECEIPT, phase}; break; }
  if (r.halt_reason)                     { report.halted={reason:r.halt_reason, phase}; break; } // incl. coord:<reason> (F3), funnel:<reason>
  if (r.receipt_status === 'ConcludeIncomplete') { report.halted={reason:HALT.CONCLUDE_INCOMPLETE, phase}; break; }
  if (r.receipt_status === 'Blocked')    { report.halted={reason:HALT.PHASE_BLOCKED, phase}; break; }  // (F4)
  if (r.receipt_status !== 'Completed')  { report.halted={reason:`${HALT.ANOMALY}:${r.receipt_status}`, phase}; break; } // Unknown lands here (F4)
  if (!r.verify.green)                   { report.halted={reason:HALT.VERIFY_RED, phase}; break; }

  ready = r.next_ready;                                // consume authority, NEVER re-derive
}
report.divergence = await divergenceProbe(slice);  // agent() → read-only divergence tool (§5.5); NOT raw JS git
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
  ANOMALY, VERIFY_RED, BUDGET_EXHAUSTED }`. Single source; the loop references
  members, never inline literals (STD-001 in the JS reference).

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
| `dispatch-orchestrator`| `dispatch_import`, `dispatch_conclude_phase`, `dispatch_reap`, `dispatch_phase_receipt`*, `dispatch_next_ready`*, `dispatch_authored_divergence`* | Read, Edit, Write, Bash, Grep, Glob, Agent (needs them to run the funnel + tests) |
| **`dispatch-probe`** (new) | `dispatch_next_ready`*, `dispatch_authored_divergence`*, `dispatch_phase_receipt`* | **Read, Grep, Glob only** — no Write/Edit/Bash/Agent |
| `dispatch-worker`      | `worker_commit`                                       | Read, Edit, Write, Bash, Grep, Glob |
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

- **Coord-root placement is load-bearing (PHASE-05 delta).** The orchestrator is
  spawned with NO `isolation`, so it inherits the driver's cwd; the driver MUST be
  invoked parked at coord-root (`.dispatch/SL-<n>` on `dispatch/<n>`). Only then is
  the orchestrator jailed to the coord tree (RO `.git`, writable coord working tree)
  and `cwd_is_coord_root` fires for the nested worker fork (SL-199). Fail-closed: the
  orchestrator's first action asserts `pwd == coord-root ∧ branch == dispatch/<slice>`
  and halts with `halt_reason="coord:not-at-coord-root"` on mismatch — a misplaced
  driver can never silently fork the wrong base. RO shared `.git` is EXPECTED, not a
  fault: every coord `.git` write is a server-side MCP funnel call, never the
  orchestrator's own git.
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
to orchestrator confinement, so an **ADR-008 amendment is in scope** (§7, §9). It
is provably safe: escalation is closed at the spawn seam with **no arming token**.
Proven end-to-end — P1 (nomination round-trip), P3 (discriminator), P4 (active
deny), 2026-07-06 (`unjail-direction.md` §6).

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
- **I1 — one list, two consumers.** The gate's privileged deny-set IS the same
  authored list as the nomination-eligible-set — never two lists that can drift.
  A type added to nomination without the gate is an open escalation hole; the
  design binds them to one source so the gate cannot lag a new nomination.
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

- **OQ-1** — Where does `/drive-slice` live as an *authored, shippable*
  artifact? Baseline: `install/`-templated → seeded to `.claude/workflows/`.
  Confirm the install path exists / is the right home (vs a harness-local file
  the slice merely documents). Resolve in phase-planning.
- **OQ-2** — Does `compute_next_phases` need a machine-readable (JSON) surface,
  or does `dispatch_next_ready` wrap it cleanly without touching the existing
  CLI rendering? (Expected: thin wrapper, no CLI change.)

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
- **R4 — SQ3 (confined fork-at-base from a workflow) not empirically re-demoed**
  — leans on SL-199 prior art. *Mitigation*: plan **phase-0** is a narrow SQ3
  de-risk (one real `/drive-slice` against a scratch slice, asserting the fork
  mints at armed base on `dispatch/<n>` end-to-end); does NOT re-run CHR-039's
  settled probes.
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

**Declared touch-set (selector accuracy, post-RV-255 correction).** The Rust
design-targets are `src/dispatch.rs` (phase_projection + ReceiptStatus),
`src/mcp_server/dispatch.rs` (the three emitter tools), `src/mcp_server/tools.rs`
(MCP registration), and `src/doctor_checks.rs` (check #9 allowlist/marker growth).
The `/drive-slice` authored home is **not yet a design-target**: no selector
covers `install/workflows/**`, and `.claude/workflows/**` is scope-relevant AND
gitignored (the installed copy only). When OQ-1 fixes the committed home, that
path MUST be registered `--intent design-target` before the script is committed,
else conformance reds it as undeclared. `src/install.rs` (probe asset + workflows
seeding leg, if the "seeded by install" claim is kept) rides the broad
`src/**` scope-relevant selector — touched but not audit-red.

**Phase-0 e2e (manual, doubles as the SQ3 demo + the `/drive-slice` inspection
verification):** one real `/drive-slice` against a scratch slice — confined fork
mints at armed base on `dispatch/<n>`; a phase drives to `Completed`; an injected
red verify halts the loop without auto-merge; the forbidden-write runtime probe
refuses.

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
