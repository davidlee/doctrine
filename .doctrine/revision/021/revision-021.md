# REV REV-021 — ADR-008: nominated-unjailed orchestrator

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-206 D8 supersedes SL-199 Mode B (confined orchestrator, every coord write via
server-side MCP): the dispatch orchestrator now runs **nominated-unjailed**
(`PassThrough`, RW `.git`). That is a new exception to the confinement boundary
this ADR prescribes, so an ADR-008 amendment is in scope (SL-206 `design.md`
§5.6, §7 D8). Chosen for **simplicity of reasoning + lower implementation
complexity, not necessity** — a confined orchestrator was proven viable (P0 Q2,
`unjail-direction.md` §6); unjail buys plain-git convenience for the orchestrator
at the price of a standing obligation, recorded below so the ADR carries the cost
honestly.

Safety proof status is **per-seam, not blanket**: the `Agent` seam is proven
closed end-to-end — P1 (nomination round-trip), P3 (`PreToolUse(Agent)` carries
the spawner id), P4 (active deny, E2E), all 2026-07-06. The `Workflow` seam is
closed by **fail-safe blanket deny** (any caller with a present `agent_id`) until
OQ-4 proves identity carriage; its outcome only decides whether that seam can be
*relaxed* to the finer identity-gate, never whether it is safe.

This revision is **proposed** and is **applied at SL-206 reconcile** — the ADR
edit is real only once the implementation lands.

### Amendment — new decision D-B6: nominated-unjailed orchestrator (SL-206)

| Where | Before | After |
|---|---|---|
| ADR-008 § Decision | Orchestrator confinement follows the general subagent posture: jailed (PreToolUse worktree-jail); SL-199 Mode B keeps the orchestrator confined with coord writes via server-side MCP | New **D-B6**: the dispatch orchestrator is **nominated-unjailed** — `SubagentStart` nomination + `PreToolUse(Agent)` spawn-gate, invariants I1/I2, fail-safe direction, and the three-item ledger below |

**Mechanism (two halves, one closed loop).**

1. **Nominate.** A `SubagentStart(dispatch-orchestrator)` hook writes the spawned
   `agent_id` to a PassThrough allowlist at a FIXED `$CLAUDE_PROJECT_DIR`-resolved
   path (never cwd-relative — hook cwd is the *spawned* tree). Sync-blocking: the
   entry lands before the orchestrator's first `PreToolUse`. `decide()` grants
   `PassThrough` scoped to that one id.
2. **Gate.** A `PreToolUse(Agent)` matcher **denies** the spawn iff
   `agent_id present ∧ caller ∉ allowlist ∧ subagent_type ∈ privileged-set`.
   The caller `agent_id` is present iff subagent-initiated (P3) — the parentage
   discriminator `SubagentStart` lacks (P0 Q1) — so a jailed `Agent`-holder cannot
   spawn an auto-nominated child; escalation closes at the source (P4). The arming
   token is retired.

**Invariants.**

- **I1 — one list, EVERY spawn seam.** The gate's privileged deny-set IS the
  nomination-eligible-set — one authored source, gating every harness spawn seam
  (`Agent`, `Workflow`, any future surface; a new seam defaults to gated, not
  forgotten). **Checked, not asserted**: a doctor check (doctor-#9 sibling) fails
  Error-severity when (a) any nominated `subagent_type` is missing from the gate
  deny-set, or (b) a registered spawn seam has no `PreToolUse` matcher at all.
- **I2 — config outside every jail.** Allowlist + hook config live at a path
  writable only by the main thread (worktree-jail Bash deny + import R-5 belt);
  a worker cannot nominate anything.
- **Fail-safe.** Every failure mode lands on *more* confinement: a missed
  `SubagentStart` (not fail-closeable) leaves the orchestrator jailed — a visible
  functional failure, never an escape. A `SubagentStop` matcher removes the
  allowlist entry (stale-id hygiene; fires awaited, tree-intact).

**Ledger — three things the amendment records beyond the grant** (design §5.6):

- **(a) Mode-B as reversible escape hatch.** The confined orchestrator loses
  nothing functional under Workflow (import already routes via the shipped MCP);
  if the seam-symmetry obligation proves unmaintainable, revert with no
  functional loss.
- **(b) Seam-symmetry as the standing price.** I1's duty is permanent for as long
  as the orchestrator runs unjailed. The toll stays one-time ONLY while it is
  conformance-checked (the I1 doctor check); as prose it would be re-paid per
  harness release.
- **(c) O-steered blast radius, named out loud.** P1/P3/P4 bound **jailed→O
  escalation** only — they say nothing about **O-steered**: the orchestrator's
  primary attacker-shaped input is the worker's diff, and a nominated-unjailed
  orchestrator has main-thread reach (primary tree, history, `.claude/`) that
  Mode-B's tunnel-set bounded. The residual is roughly what the main thread
  already accepts running `/dispatch` by hand, mitigated by trusted committed
  def, narrow prompt, structured inputs (diffs + typed receipts, not free-form
  instructions) — but stated as a sentence, not implied covered by the
  escalation proofs.

### Unchanged

- **D-B1…D-B5 stand untouched.**
- **N1** (`worker_commit` sanctioned exception) stands; the same spawn-gate
  additionally hardens the shipped `dispatch-worker` provisioning matcher
  (escalatable today, bounded).
- Rule recorded alongside: **grant power scales with intent evidence** —
  type-match alone buys bounded/gated capability (worker tier); anything
  unbounded (`PassThrough`) requires the spawn-gate.

### Consistency

ADR-006 / CHR-039: the orchestrator is unconfined *by design* (sole writer).
Nomination relocates the main thread's existing unjailed status to a trusted
committed def — the def name cannot be forged (`.claude/agents/` is
worker-unwritable: jailed + R-5) — it invents no new trust.
