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

- **Rust-primary, narrow.** The only new Rust is a **read-only** phase-receipt
  emitter + a `next_ready` surface; the `/drive-slice` script is an
  install-templated reference, not `cargo test`-able.
- **DRY over the readiness/status authorities** — no parallel state machine in
  JS; consume `compute_next_phases` and a single per-phase projection.
- **Durable truth over disposable truth** — receipt status is sourced from the
  committed boundary ledger, not the gitignored sheet.
- **Containment by grant, not by hope** — the safety boundary is the subagent
  tool allowlist, checked (static + runtime), not assumed.
- **Report-and-halt** — the driver never crosses the authored split-brain and
  never auto-merges (SL-199 §D intact).

## 5. Proposed Design

### 5.1 System Model

```
Workflow /drive-slice   (JS, harness; args:{slice}; budget pool = live, tree-wide)
  ├─ bootstrap: planner agent() → next_ready[]        (compute_next_phases authority)
  └─ loop while next_ready non-empty & budget allows:
       agent(dispatch-orchestrator, isolation:'worktree', schema: PhaseReceipt)
          ├─ arm base → dispatch-worker(fork) → import → conclude → reap  [SL-199 as-shipped]
          ├─ call dispatch_phase_receipt{slice,phase}    [NEW, read-only]
          ├─ call dispatch_next_ready{slice}             [NEW, read-only]  (slice-global)
          └─ graft verify{} + worker_branch; attach next_ready[]
          ▶ returns PhaseReceipt (schema-validated)
       JS: receipt.receipt_status / halt_reason → advance | retry | halt+report
```

Two trust tiers (unchanged from SL-199): the **workflow** is a dumb sequencer
(no writes, no arming, no MCP); the **orchestrator** is the sole writer, drives
the funnel via its narrow grant.

### 5.2 Interfaces & Contracts

**The emitter (new Rust, read-only).** Two surfaces over one reader:

- CLI: `doctrine dispatch phase-receipt --slice <N> --phase <PHASE-NN>` → typed JSON
- MCP: `dispatch_phase_receipt{slice, phase}` (granted to `dispatch-orchestrator`)

```rust
// durable, boundary-ledger-backed — the receipt's load-bearing truth
enum ReceiptStatus {
    NotStarted,          // no sheet progress, no boundary row
    InProgress,          // sheet in_progress, no boundary row
    Completed,           // boundary row exists for the phase on dispatch_tip (DURABLE)
    ConcludeIncomplete,  // sheet=completed ∧ boundary missing — retryable funnel fault (§5.4)
}

struct PhaseReceiptCore {
    slice: u32,
    phase: String,                       // PHASE-NN (immutable id)
    receipt_status: ReceiptStatus,       // durable, boundary-backed
    runtime_status: Option<SheetStatus>, // advisory, sheet-derived, nullable
    dispatch_tip: String,                // dispatch branch HEAD (NOT a code oid)
    boundary: Option<Boundary>,          // Some ⟺ boundary row exists
    coord_error: Option<CoordRefusal>,   // unknown-slice | ambiguous | stale (resolve_coord)
}
struct Boundary { code_start: String, code_end: String }  // code-range OIDs, distinct from dispatch_tip
enum CoordRefusal { UnknownSlice, Ambiguous, Stale }
```

`ReceiptStatus` is a **new** enum, NOT `state::PhaseStatus` (that is a sheet
lifecycle; reusing it would let callers infer durable guarantees from advisory
`runtime_status`). `dispatch_tip` and `boundary.code_end` are **named by role**
— both are commit OIDs but different tips; never a generic `coord_tip`.

**Readiness (new Rust, read-only, slice-global).**
`dispatch_next_ready{slice}` → `{ next_ready: Vec<String> }` — a thin wrapper
over the **existing** `compute_next_phases` authority. Deliberately a separate
surface from the phase receipt (see D3): "what happened to phase X" and "what
should the slice do next" are different altitudes.

**The full phase receipt (harness-side `schema:`; orchestrator-composed).** The
orchestrator grafts its runtime facts onto the emitter core; the `schema:` on
the `agent()` call forces the shape and the harness validates it:

```
PhaseReceipt = PhaseReceiptCore                        // (from dispatch_phase_receipt)
  + worker_branch : string                             // orchestrator armed/spawned it (ephemeral, reaped)
  + verify        : { green: bool, failures: string[] }// orchestrator RAN the tests
  + halt_reason?  : string                             // set on any stop
  + next_ready    : string[]                           // slice-global adjunct (from dispatch_next_ready) — labelled, not part of the durable core
```

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

**Driver loop (JS, `/drive-slice`):**

```js
const SEED_PHASE_COST = 45_000;   // RFC-011-observed funnel ceremony (STD-001: rationale in comment)
const SOFT_CEILING    = 120_000;  // advisory dumb-zone; planning-only, NEVER gated

let lastActual = null;
const report = { phases: [], halted: null, divergence: null };
// NB: the script has no shell / no MCP — every doctrine read is via a spawned
// agent. `planner`/`divergenceProbe` below are lightweight agent() calls granted
// only the relevant read-only tool, NOT raw JS.
let ready = await planner(slice);   // agent() → dispatch_next_ready{slice} (compute_next_phases authority)

while (ready.length) {
  const phase = ready[0];
  const est = lastActual ?? SEED_PHASE_COST;
  if (budget.total && budget.remaining() < est) { report.halted = {reason:'budget-exhausted', phase}; break; }

  const before = budget.spent();
  const r = await agent(orchestratorPrompt(slice, phase),
                        { isolation:'worktree', schema: PhaseReceipt });
  lastActual = budget.spent() - before;               // adaptive
  report.phases.push(r);
  log(`phase ${phase}: ${lastActual/1000|0}k; soft-ceiling headroom ~${((SOFT_CEILING-lastActual)/1000)|0}k`); // advisory

  if (!r)                                { report.halted={reason:'null-receipt', phase}; break; }
  if (r.coord_error)                     { report.halted={reason:`coord:${r.coord_error}`, phase}; break; }
  if (r.halt_reason)                     { report.halted={reason:r.halt_reason, phase}; break; }
  if (r.receipt_status === 'ConcludeIncomplete') { report.halted={reason:'funnel:conclude-incomplete-retryable', phase}; break; }
  if (r.receipt_status !== 'Completed')  { report.halted={reason:`anomaly:${r.receipt_status}`, phase}; break; }
  if (!r.verify.green)                   { report.halted={reason:'verify-red', phase}; break; }

  ready = r.next_ready;                                // consume authority, NEVER re-derive
}
report.divergence = await divergenceProbe(slice);  // agent() → read-only divergence tool (§5.5); NOT raw JS git
return report;
```

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

| Subagent type          | MCP grant (exhaustive)                                                             |
|------------------------|-----------------------------------------------------------------------------------|
| `dispatch-orchestrator`| `dispatch_import`, `dispatch_conclude_phase`, `dispatch_reap`, `dispatch_phase_receipt`*, `dispatch_next_ready`*, `dispatch_authored_divergence`* |
| `dispatch-worker`      | `worker_commit`                                                                   |
| workflow script        | *(none — JS, no MCP)*                                                             |

(* new, read-only.) The bootstrap planner + closing divergence probe are
`dispatch-orchestrator` spawns (they need only the read-only tools). The three
new tools add **no write surface**. The gate is
checked two mechanically-real ways (no runtime grant-readback API exists — D2):
- **Static** — phase-0 inspects the authored subagent-definition tool allowlists
  (`install/agents/claude/dispatch-orchestrator.md`,
  `install/agents/claude/dispatch-worker.md` — exact files in §9) for exactly the
  intended set. These files are in the design-target selectors, so `slice
  conformance` holds them.
- **Runtime** — phase-0 has the orchestrator attempt a forbidden
  `review_*`/`memory_*` write → confirmed absent/refused; and confirms positive
  callability of the granted funnel tools.

### 5.5 Invariants, Assumptions & Edge Cases

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
  where `trunk_ref` is resolved from the **existing dispatch trunk authority**
  (`src/dispatch.rs:630`), NOT hardcoded `edge`, and recorded in `compared_ref`
  so the signal is not repo-local folklore. IMP-174 owns the partition /
  refuse-loud semantics. **Because the script has no shell**, this is a
  **read-only doctrine tool** (`dispatch_authored_divergence{slice}`, granted to
  the closing probe agent), not raw JS `git` — it rides the same coord-resolution
  + trunk authority as the emitter. (Adds a third read-only tool; no write
  surface — folded into the same grant-gate story as §5.4.)
- **`resolve_coord` refusals are workflow halt states**, not noise: `stale`
  (torn-down/prunable coord) and `ambiguous` are plausible from the primary tree
  and must halt loudly.

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

## 9. Quality Engineering & Validation

**Unit (Rust, TDD red/green/refactor):**
- `phase_projection` — per-phase status derivation: boundary-present ⇒
  `Completed`; sheet-completed ∧ boundary-missing ⇒ `ConcludeIncomplete`;
  in_progress/none ⇒ `InProgress`/`NotStarted`. Table-driven over fixture coord
  trees (ride existing dispatch test rig).
- `dispatch_phase_receipt` — `coord_error` surfaced for each `resolve_coord`
  refusal; `dispatch_tip` vs `boundary.code_end` distinctness.
- `dispatch_next_ready` — agrees with `compute_next_phases` on the same fixtures.
- `dispatch_authored_divergence` — `diverged` true iff `.doctrine/**` differs
  `trunk_ref..dispatch_tip`; `compared_ref` = the resolved trunk authority (not
  hardcoded `edge`); read-only, no coord mutation.
- **Behaviour-preservation**: existing `dispatch status` suite green **unchanged**
  after the `phase_projection` extract.

**Static / conformance:**
- The subagent-def allowlist files (exact paths, recorded as design-target
  selectors): `install/agents/claude/dispatch-orchestrator.md`,
  `install/agents/claude/dispatch-worker.md` — asserted to grant exactly the
  intended tool sets (+ the two new read-only tools on the orchestrator). The
  installed instance under `.claude/agents/` is regenerated by `doctrine install`.

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
Offer `/inquisition` or an external hostile pass before `/plan`.
