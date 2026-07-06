// SPDX-License-Identifier: GPL-3.0-only
//
// /drive-slice — the doctrine dispatch reference workflow (SL-206).
//
// A budget-adaptive driver that walks one slice's ready phases to completion by
// spawning ONE confined `dispatch-orchestrator` per phase, consuming the typed
// PhaseReceipt each returns, and halting loudly on any non-Completed signal. It
// is a DUMB SEQUENCER: it holds no shell, no git, and no MCP — every doctrine
// read is delegated to a spawned agent granted only the relevant read-only tool
// (the `dispatch-probe` role). It NEVER arms a base, NEVER imports, NEVER
// concludes, NEVER merges, and NEVER crosses the authored split-brain — landing
// the fork's deltas stays with `/audit -> /reconcile -> /close` (IMP-174).
//
// Transcribed from `.doctrine/slice/206/design.md` §5.4 — the driver loop is the
// authored source; this file must not drift from it.
//
// PHASE-06 corrections (design §5.4 delta `ef23828f`, placement reconciliation):
//   - PLACEMENT: the per-phase orchestrator `agent()` carries NO `isolation`. A
//     no-isolation subagent starts in the driver's cwd — the slice's coordination
//     worktree root — where the SL-199 arming discriminator fires. `isolation:
//     'worktree'` fresh-forked it into a detached tree where it never armed, so no
//     phase ever reached Completed. The confined-orchestrator model is proven
//     (SL-199 Mode B); the defect was placement, not permission.
//   - F1 (harness contract): `meta` is a pure literal (single-string description,
//     `[{title}]` phases); the drive logic runs in a TOP-LEVEL body — the Workflow
//     tool executes the body and never calls a `run` export.
//   - F2 (slice guard): the top-level body tolerates args-as-string and fails
//     closed on a non-positive-integer slice, so the driver never guesses which
//     slice it drives.

export const meta = {
  name: 'drive-slice',
  description:
    "Drive one slice's ready phases to completion via one confined dispatch-orchestrator per phase; consume typed PhaseReceipts, report-and-halt, never auto-merge. Read-only bootstrap + closing divergence probe via dispatch-probe.",
  phases: [{ title: 'bootstrap' }, { title: 'drive' }, { title: 'divergence-probe' }],
};

// ── Budget config (STD-001: named, rationale in comment; in-script per D4) ──
// These are HARNESS-side JS constants consumed by `budget.*`; the Rust emitter
// never sees them. No new doctrine config surface.
const SEED_PHASE_COST = 45_000; // RFC-011-observed funnel ceremony seed cost
const SOFT_CEILING = 120_000; // advisory dumb-zone; planning-only, NEVER gated

// ── Halt-reason vocabulary (F6): a single-sourced, closed contract ──
//
// Two families of halt reason. The loop branches on them AS PROTOCOL, so the
// vocabulary must never be scattered inline literals (STD-001):
//
//  1. Rust-derived, re-exported (NOT re-invented here). The orchestrator forwards
//     these on `receipt.halt_reason`; the driver passes them through verbatim:
//       - `funnel:<reason>` — from `FunnelOutcome::Refused.reason`
//         (src/mcp_server/dispatch.rs: `FunnelOutcome` @ ~:351,
//          `funnel_refused(reason, detail)` @ ~:363).
//       - `coord:<reason>`  — from `CoordRefusal`
//         (src/mcp_server/dispatch.rs: `CoordRefusal` @ ~:52, reasons
//          `unknown-slice` | `ambiguous` | `stale`).
//     The design does not mint these strings; it forwards the authored enums.
//     ONE exception, minted at the orchestrator (not the emitter): the
//     fail-closed placement precondition below emits `coord:not-at-coord-root`
//     — an orchestrator-detected extension of the `coord:` family (a
//     coordination-worktree fault), distinct from the Rust `CoordRefusal` enum.
//     The driver passes it through verbatim like any other `coord:` reason.
//
//  2. Script-local — the driver's ONLY authored halt vocabulary. Members only,
//     never inline literals.
const HALT = Object.freeze({
  BUDGET_EXHAUSTED: 'budget-exhausted',
  NULL_RECEIPT: 'null-receipt',
  CONCLUDE_INCOMPLETE: 'conclude-incomplete',
  PHASE_BLOCKED: 'phase-blocked',
  ANOMALY: 'anomaly',
  VERIFY_RED: 'verify-red',
});

// ── PhaseReceipt schema (design §5.4) ──
//
// PhaseReceipt = PhaseReceiptCore (from `dispatch_phase_receipt` Resolved; ABSENT
// on CoordRefused) + orchestrator-supplied runtime facts. Passed as `schema:` on
// the per-phase `agent()` call so the harness validates the returned shape.
//
//   PhaseReceiptCore = { slice, phase, receipt_status, runtime_status?,
//                        dispatch_tip, boundary? }
//   + worker_branch : orchestrator armed/spawned it (ephemeral, reaped)
//   + verify        : orchestrator RAN the tests
//   + halt_reason?  : set on any stop (incl. `coord:<reason>` / `funnel:<reason>`)
//   + next_ready    : slice-global adjunct (from `dispatch_next_ready`), labelled
const PhaseReceipt = {
  type: 'object',
  additionalProperties: false,
  // On a CoordRefused emitter result the orchestrator emits a MINIMAL receipt
  // (`halt_reason` only, no core fields), so only `halt_reason` is required (F3).
  required: ['halt_reason'],
  properties: {
    slice: { type: 'integer' },
    phase: { type: 'string' }, // PHASE-NN (immutable id)
    receipt_status: {
      type: 'string',
      enum: [
        'NotStarted',
        'InProgress',
        'Blocked',
        'Completed',
        'ConcludeIncomplete',
        'Unknown',
      ],
    },
    runtime_status: { type: ['string', 'null'] }, // advisory, sheet-derived
    dispatch_tip: { type: 'string' }, // dispatch branch HEAD (NOT a code oid)
    boundary: {
      type: ['object', 'null'],
      additionalProperties: false,
      properties: {
        code_start: { type: 'string' },
        code_end: { type: 'string' }, // code-range OID, distinct from dispatch_tip
      },
    },
    worker_branch: { type: 'string' }, // ephemeral fork branch, reaped
    verify: {
      type: 'object',
      additionalProperties: false,
      required: ['green', 'failures'],
      properties: {
        green: { type: 'boolean' },
        failures: { type: 'array', items: { type: 'string' } },
      },
    },
    halt_reason: { type: ['string', 'null'] },
    next_ready: { type: 'array', items: { type: 'string' } },
  },
};

// ── Spawned read-only helpers (dispatch-probe role) ──
// The script has no MCP; these delegate the two doctrine reads to a genuinely
// read-only agent (Read/Grep/Glob + the three read-only tools ONLY).

async function planner(slice) {
  // Bootstrap: report the ready phase batch (the compute_next_phases authority,
  // verbatim, via dispatch_next_ready).
  const r = await agent(
    `Call dispatch_next_ready for slice ${slice} and report the ready phase ids in order.`,
    {
      agentType: 'dispatch-probe',
      schema: { type: 'object', required: ['next_ready'], properties: { next_ready: { type: 'array', items: { type: 'string' } } } },
    },
  );
  return r.next_ready;
}

async function divergenceProbe(slice) {
  // Closing: report the .doctrine/** divergence advisory as RAW signal only —
  // never gated, never acted on (IMP-174).
  return agent(
    `Call dispatch_authored_divergence for slice ${slice} and report {diverged, compared_ref, drifted_paths?} verbatim. Do not act on it.`,
    {
      agentType: 'dispatch-probe',
      schema: {
        type: 'object',
        required: ['diverged', 'compared_ref'],
        properties: {
          diverged: { type: 'boolean' },
          compared_ref: { type: 'string' },
          drifted_paths: { type: 'array', items: { type: 'string' } },
        },
      },
    },
  );
}

function orchestratorPrompt(slice, phase) {
  return (
    `You are driving slice ${slice} phase ${phase} through the dispatch funnel from ` +
    `inside its coordination worktree.\n\n` +
    `PLACEMENT PRECONDITION (fail-closed — assert FIRST, before arming, spawning, ` +
    `importing, or concluding anything). Run \`pwd\` and \`git branch --show-current\`. ` +
    `You MUST be at the slice's coordination worktree root (the .dispatch/SL-${slice} ` +
    `tree) on branch dispatch/${slice}. If pwd is NOT the coord root OR the branch is ` +
    `NOT dispatch/${slice} — e.g. you were fresh-forked into a detached agent worktree ` +
    `where the SL-199 coord-root arming discriminator never fires — DO NOT arm, spawn, ` +
    `import, or conclude. Immediately return a MINIMAL receipt with ` +
    `halt_reason="coord:not-at-coord-root" and stop.\n\n` +
    `Only when placement holds: arm the base, spawn a worker to execute the phase, ` +
    `import the delta, conclude the phase, reap the fork, then compose and return a ` +
    `PhaseReceipt: source its durable core from dispatch_phase_receipt, its readiness ` +
    `adjunct from dispatch_next_ready, and run the verify command — set verify.green + ` +
    `failures from the result. On ANY funnel refusal, set halt_reason="funnel:<reason>" ` +
    `and return a non-Completed receipt; on a coord refusal, return a minimal receipt ` +
    `with halt_reason="coord:<reason>". Never auto-merge, never land to trunk.`
  );
}

// ── Driver loop (JS, /drive-slice) — transcribed from design §5.4 ──

async function run({ slice }) {
  let lastActual = null;
  const report = { slice, phases: [], halted: null, divergence: null };

  let ready = await planner(slice); // compute_next_phases authority

  while (ready.length) {
    const phase = ready[0];
    const est = lastActual ?? SEED_PHASE_COST;
    // null `total` → unmetered: `budget.total &&` short-circuits (D4-a).
    if (budget.total && budget.remaining() < est) {
      report.halted = { reason: HALT.BUDGET_EXHAUSTED, phase };
      break;
    }

    const before = budget.spent();
    // PLACEMENT (PHASE-06 fix, design §5.4 delta): NO isolation. A no-isolation
    // subagent starts in the driver's current working directory
    // (docs/claude/subagents.md:263) — the slice's coordination worktree root,
    // where the SL-199 arming discriminator (cwd_is_coord_root ∧ coord_in_dispatch,
    // src/worktree/create.rs) fires and the orchestrator arms/spawns/imports through
    // the server-side MCP funnel over the RO shared .git (integrity never rests on
    // the confined orchestrator; SL-199 Mode B). isolation:'worktree' would
    // FRESH-FORK it into a detached .worktrees/agent-<hex> where the discriminator
    // never fires and no phase ever reaches Completed.
    const r = await agent(orchestratorPrompt(slice, phase), {
      schema: PhaseReceipt,
      agentType: 'dispatch-orchestrator',
    });
    lastActual = budget.spent() - before; // adaptive cost (D4-i)
    report.phases.push(r);
    // Advisory dumb-zone (log-only, NEVER gates a one-per-phase run).
    log(
      `phase ${phase}: ${(lastActual / 1000) | 0}k; soft-ceiling headroom ~${((SOFT_CEILING - lastActual) / 1000) | 0}k`,
    );

    // Halt vocabulary is the NAMED, single-sourced contract above (F6). Every
    // branch references a HALT.* member or passes through receipt.halt_reason
    // verbatim (the Rust-derived coord:/funnel: cases) — never a bare literal.
    if (!r) {
      report.halted = { reason: HALT.NULL_RECEIPT, phase };
      break;
    }
    if (r.halt_reason) {
      // Includes coord:<reason> (F3) and funnel:<reason> — Rust-derived, verbatim.
      report.halted = { reason: r.halt_reason, phase };
      break;
    }
    if (r.receipt_status === 'ConcludeIncomplete') {
      // Retryable funnel fault, but the repair is the ORCHESTRATOR's (retry
      // conclude against the live dispatch_tip) — the driver never blind-retries.
      report.halted = { reason: HALT.CONCLUDE_INCOMPLETE, phase };
      break;
    }
    if (r.receipt_status === 'Blocked') {
      report.halted = { reason: HALT.PHASE_BLOCKED, phase }; // (F4)
      break;
    }
    if (r.receipt_status !== 'Completed') {
      // Unknown (malformed/unreadable sheet, fail-loud) lands here too (F4).
      report.halted = { reason: `${HALT.ANOMALY}:${r.receipt_status}`, phase };
      break;
    }
    if (!r.verify.green) {
      report.halted = { reason: HALT.VERIFY_RED, phase };
      break;
    }

    ready = r.next_ready; // consume the authority, NEVER re-derive
  }

  // Closing divergence advisory — raw signal only, never gated, never acted on.
  report.divergence = await divergenceProbe(slice);
  return report;
}

// ── Top-level entry (F1: the Workflow tool executes this body; it never calls a
// `run` export). F2 slice guard: tolerate args-as-string (the tool's documented
// footgun) AND fail closed on a non-positive-integer slice — the driver must
// never guess which slice it drives. ──
const parsed = typeof args === 'string' ? JSON.parse(args) : (args || {});
const slice = Number(parsed.slice);
if (!Number.isInteger(slice) || slice < 1) {
  throw new Error(
    `drive-slice refuses: expected a positive integer slice id, got ${JSON.stringify(parsed.slice)}`,
  );
}
log(`drive-slice: starting for slice ${slice}`);
const result = await run({ slice });
log(`drive-slice: report ${JSON.stringify(result).slice(0, 800)}`);
return result;
