# Review RV-251 — reconciliation of SL-199

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** SL-199 — confined dispatch-orchestrator capstone: a plain (non-
worktree) subagent drives one slice's phases from inside its coordination worktree
through three privileged funnel MCP tools (`dispatch_import`,
`dispatch_conclude_phase`, `dispatch_reap`), reads-raw / writes-MCP wall, nested
workers self-commit via `worker_commit`.

**Review surface.** Dispatched slice (ADR-007 R2): `review/199` + `phase/199-0N`
are immutable evidence refs. Audit ran against the **candidate interaction branch**
`candidate/199/review-001` = `main` (v0.15.7, 1fbb9c3d) ⊕ `review/199` impl bundle,
materialized at `.doctrine/state/dispatch/candidate/cand-199-review-001`. The
auto-merge conflicted on a single authored append-log (`.doctrine/rfc/011/
case-notes.md`) — both edge and the fork appended disjoint RFC-011 notes; resolved
by union (merge 3294fbbf). All code merged clean.

**Lines of attack.**
1. Does the implementation match the (re-locked, inquisition-reconciled) design —
   the confined Fork trigger, the funnel tools, the reads-raw/writes-MCP wall?
2. Path-conformance: does what git touched match the `design-target` selectors?
3. Does the project gate (`doctrine check gate` → `just validate`) pass on the
   merged surface?
4. Is VH-1 (live e2e witness) genuinely satisfied, and verify-vt green?

**Held-to invariants.** VH-1 witnessed + accepted (prior turn); verify-vt VT gate
green on `dispatch/199` (all 5 phases, 15 VT criteria PASS); prepare-review
completeness gate satisfied (registry 5/5 after primary phase-status sync).

## Synthesis

**Closure story.** SL-199's implementation is correct and matches its design. The
capstone — a confined dispatch-orchestrator driving a real phase fork→land→
conclude→reap with the raw-`.git` wall intact and nested confinement holding — was
witnessed live (VH-1, accepted prior turn). The VT existence/shape gate is green
across all five phases (15 `VT` criteria PASS, run against `dispatch/199` — the
delta-bearing tree; a run against `edge` falsely fails because edge carries none of
the delta). The SL-199-built `doctrine doctor` is **exit 0** on the merged
candidate: the role-keyed agent-conformance check (PHASE-04) blesses the
orchestrator's three funnel tokens exactly as its own unit test asserts.

**The one loud signal — gate red — is a measurement artifact (F-1).** `just
validate` calls bare `doctrine doctor`, which resolves to the *installed* binary
`~/.cargo/bin/doctrine` v0.15.7, built from `main` **before** SL-199. That stale
binary still carries the OLD **role-blind** agent-conformance check, which forbids
any confined agent from holding anything but `worker_commit` — so it flags the
orchestrator's `dispatch_import` / `dispatch_conclude_phase` / `dispatch_reap`. This
is the self-tightening-check bootstrap: the very slice that upgrades a doctor check
also ships the artifact the upgrade blesses, and the installed check cannot see its
own upgrade until integrate + reinstall. The slice's own code gate is green. Same
class as T7. **Resolves deterministically on `/close` integrate + `cargo build` +
`doctrine install`.**

**Conformance.** `slice conformance 199` surfaced a selector-registry hygiene gap,
not scope creep (F-2, F-3, F-4). The registry under-declares real deliverables
(the 1384-line `src/mcp_server/dispatch.rs` funnel server, `src/dispatch.rs`,
`src/doctor_checks.rs`, the test files, the install-side agent defs + orchestrator
hymn) and mis-targets the materialized `.claude/agents/dispatch-orchestrator.md`
(work landed on the `install/agents/claude/` source). Authored governance
(`.doctrine/slice/199/*`, `case-notes.md`) appears as undeclared noise because
`record-boundary` captures commit ranges, not path-filtered code. None is a
behavioural defect; all route to the selector registry via `/reconcile`.

**Standing risks / tradeoffs consciously accepted.**
- The gate cannot go green **before** integrate — inherent to a self-tightening
  doctor check. Operator MUST refresh the installed binary post-integrate; until
  then `just validate` reads red for a non-defect.
- Two conclude-cadence tooling gaps hit while concluding (recorded to RFC-011
  case-notes, `sl199-conclude`): (a) primary/coord phase-status split-brain —
  prepare-review's completeness gate reads the completed-set from the primary tree
  while dispatch flips live in the coord runtime, and the documented fix
  `reconcile-phases` refuses while the coord tree exists (idea/028); resolved here
  by a direct primary phase-status sync. (b) `verify-vt` run post-coord-removal
  against `edge` reads a false FAIL — the gate scans the working tree, not a
  delta-bearing ref.

## Reconciliation Brief

### Per-slice (direct edit)

- **`slice-199.toml` selector registry (F-2)** — load-bearing: `doctrine slice
  selector add` the undeclared deliverable paths so conformance reads clean:
  `src/mcp_server/dispatch.rs`, `src/dispatch.rs`, `src/doctor_checks.rs`,
  `src/mcp_server/mod.rs`, `src/worktree/mod.rs`, `tests/e2e_mcp_server.rs`,
  `tests/e2e_worktree_create_fork.rs`, `install/agents/claude/dispatch-orchestrator.md`,
  `install/agents/claude/dispatch-worker.md`, `install/hymns/role/orchestrator.md`.
  `design.md §6` is the human mirror — cite, don't rely on it (prose alone leaves
  conformance red).
- **`slice-199.toml` selector registry (F-3)** — `doctrine slice selector rm
  .claude/agents/dispatch-orchestrator.md` (materialized copy, never authored), or
  repoint to `install/agents/claude/dispatch-orchestrator.md`. Mirror in `design.md
  §6`.

### Governance/spec (REV)

- None. No ADR / policy / standard / spec / requirement change is owed by the
  audit. (SL-199 already authored its own REQ-335 + SPEC-021 delta in-bundle;
  those are the slice's deliverables, not reconcile targets.)

### Operational follow-up (not a reconcile surface)

- **F-1** — post-integrate, `cargo build && doctrine install` to refresh
  `~/.cargo/bin/doctrine` so `just validate` reflects the role-keyed check. Track as
  the `/close` integrate step, not a spec/governance edit.

## Reconciliation Outcome

### Direct edits applied (selector registry — slice-199.toml)
- **F-2** — `slice selector add 199 --intent design-target` for 10 undeclared
  deliverables: `src/mcp_server/dispatch.rs`, `src/dispatch.rs`,
  `src/doctor_checks.rs`, `src/mcp_server/mod.rs`, `src/worktree/mod.rs`,
  `tests/e2e_mcp_server.rs`, `tests/e2e_worktree_create_fork.rs`,
  `install/agents/claude/dispatch-orchestrator.md`,
  `install/agents/claude/dispatch-worker.md`, `install/hymns/role/orchestrator.md`.
- **F-3** — `slice selector rm 199 .claude/agents/dispatch-orchestrator.md`
  (materialized mirror; the authored source `install/agents/claude/…` now carries
  the design-target, added under F-2).
- **Result:** `slice conformance 199` → undelivered 0, conformant 13. No prose
  mirror edit — this design carries no §6 selector list (design §6 is the
  feasibility probe).

### REVs completed
- None. Governance/spec brief section was empty — SL-199 authored its own
  REQ-335 + SPEC-021 delta in-bundle; nothing owed to reconcile.

### Withdrawn / tolerated
- **F-1** (tolerated) — gate-red is the stale-installed-binary artifact; resolves
  on `/close` integrate + `cargo build` + `doctrine install`. Root cause captured
  as IMP-270.
- **F-4** (tolerated) — the residual 7 `undeclared` conformance rows are all
  authored governance (`.doctrine/slice/199/*`, `case-notes.md`); design-targets
  never declare a slice's own authored files. Accepted registry-granularity noise.

Reconcile pass complete — handoff to /close.
