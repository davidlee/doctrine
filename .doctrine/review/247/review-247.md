# Review RV-247 — reconciliation of SL-198

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Reviewed surface.** The candidate interaction branch `candidate/198/review-001`
(tip `c89b124a`) — a no-ff 3-way merge of the impl bundle `review/198` (tip
`16d65fe2`) onto current trunk `refs/heads/main` (`fd49b976`). The raw
`dispatch/198` coord branch and the `phase/198-NN` cuts are immutable evidence
refs (R2); the audit runs against the merged candidate per ADR-012.

**Lines of attack.**
1. **Conformance algebra** — `slice conformance 198` after recording the late
   PHASE-04 delta: interrogate every `undeclared` (scope creep / missed design
   update) and `undelivered` (dropped work / stale design-target) cell.
2. **Design/plan text truth** — do design §5.2/§5.3/§5.4 and the plan EX/VT
   criteria still describe what shipped, after the codex X1–X5 reframe and the
   owner steers (deny-by-default, two-tier scope belt, cheap-first belts,
   authored-tree lint scope)?
3. **Governance** — is the sanctioned `worker_commit` MCP write a jail-model
   change requiring an ADR REV, or a lint-guarded exception needing only a note?
   (F4 / VH-1.)
4. **Integration soundness** — does the impl bundle merge onto current trunk and
   stay green (`check gate`), given the 26-commit base-staleness gap?

**Invariants held.** RSK-225 mitigation is the lint (jail-wall completeness);
deny-by-default is deliberate (plan EX-2); the `.doctrine/**` code floor is
fail-closed; import still refuses to promote an undeclared path; the subprocess
arm + main-thread fallback stay behaviour-preserved.

## Synthesis

SL-198 delivers the Mode B foundation — the gated `worker_commit` MCP keystone
(PHASE-02), the pure-reuse commit-import switch on the claude arm (PHASE-03), the
per-worktree dispatch record + resolver (PHASE-01), and the worker tool-surface
conformance lint that is the RSK-225 mitigation (PHASE-04, doctor check #9,
Error-gated). The integrated candidate builds clean and `doctrine check gate`
exits 0; VT-1..6 across the four phases are green, VH-1 is owner-ratified, and
VA-1 (belt genuinely rejects a `.doctrine`/`.claude`/out-of-selector write) is
satisfied by the `worker_commit.rs` belt tests plus the shared `classify_import`
reuse (VT-3) rather than a fresh live run.

**The audit found no code defects.** All eleven findings are design-artifact
truth corrections or benign process/integration observations — consistent with a
slice whose design was reframed mid-flight by the external codex passes (X1–X5)
and a sequence of owner steers that the design prose was never retro-fitted to.
The implementation consistently tracked the *plan* (the later, owner-steered
authority); the *design* text is what drifted. Nine findings (F-1..F-9) are
delegated to `/reconcile` as per-slice design edits or a governance note; F-10
(base-staleness merge conflict) was resolved in-audit as a union of concurrent
instrumentation appends; F-11 (missing `phase/198-04` cut ref) is cosmetic with
the PHASE-04 content fully present in the bundle.

**Standing risks / consciously accepted.**
- **RSK-226** (caller-binding residual — a sibling-name spoof commits on another
  live agent's branch, leaving the spoofer's own work unpromoted) is accepted and
  carried to the SL-199 capstone; the lint bounds the tool surface but not caller
  identity.
- **Deny-by-default breadth** (VH-1): every unmarked frontmatter def in the scan
  roots fails, including benign downstream non-worker agent-defs. Owner-ratified
  as the correct conservative stance (the lint IS the security mitigation).
- **Base-staleness recurrence**: `dispatch/198` forked 26 commits behind trunk;
  the candidate 3-way merge absorbed it cleanly, but the pattern keeps recurring
  (the pre-dispatch `git fetch . edge:main` ritual is the mitigation).

## Reconciliation Brief

### Per-slice (direct edit — design.md / plan)
- **design §5.2** (F-1): lint scans authored `install/agents/**` +
  `.doctrine/agents/**`, not the installed `.claude/agents` copy; update the
  design-target selectors — add the doctor host (`src/finding.rs`,
  `src/doctor_checks.rs`, `src/commands/doctor.rs`), drop/replace the
  `.claude/agents/**` selector.
- **design §5.2** (F-2): allow-by-marker → **deny-by-default** (unmarked def in
  scan roots = FAILURE; allowlist = `mcp__doctrine__worker_commit` only).
- **design §5.4 / plan PHASE-03 EX-4/EX-5** (F-3): the two-arm orchestration note
  home is `install/dispatch-mechanics.md` (generic) + `.doctrine/governance.md`
  (pi-arm), not `CLAUDE.md # orchestration`.
- **design §5.3** (F-4): record path `jail/<name>.toml` → `record/<name>.toml`.
- **design §5.2** (F-5): renumber the belt to cheap-first (scope pre-fmt → mutating
  gate; stage-by-path after fmt), per INV-5/PIN-3/X4.
- **design line 28 / plan PHASE-03 EX-2** (F-6): soften "relax `run_verify_worker`
  (code change)" to "reuse existing `is-ancestor` semantics — no production change;
  guard test in `tests/e2e_worktree_verify_worker.rs`" (VT-1 test_file pin is
  immutable — annotate, do not renumber).
- **design-target selectors** (F-7): add `src/worktree/dispatch_record.rs`; name
  `src/worktree/mod.rs` or accept a `src/worktree/**` widening for the re-export seam.
- **design-target selectors** (F-8): drop `src/dispatch.rs` (context ref, never an
  edit site — superseded by the X2 `create.rs` record).

### Governance/spec (note, not REV)
- **ADR-008 (and/or SL-182)** (F-9): record a NOTE that `worker_commit` is a
  deliberate, lint-guarded exception to the PreToolUse jail wall — structurally
  un-jails nothing (rides the witnessed MCP passthrough, RSK-225 discharged
  `7bd21f49`). The ADR-012 REV + ADR-011 D6 amendment remain **SL-199** scope, not
  this slice. VH-1 allowlist semantics owner-confirmed 2026-07-04.

### No action (recorded for completeness)
- **F-10** — base-staleness merge conflict resolved in candidate tip `c89b124a`
  (union of instrumentation appends); no code change.
- **F-11** — `phase/198-04` ref absent (PHASE-04 delta recorded late); content
  intact in the bundle. Process lesson harvested to `notes.md`.
