# Review RV-181 — reconciliation of SL-163

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed.** Solo (non-dispatch) slice; evidence read from branch
`sl-163`, promoted into the parent tree (edge) via `git merge --no-ff sl-163`
before opening this RV (review verbs refuse a worktree fork, IMP-024). Two
implementation commits audited: `2174fe6f` (the verb), `eccc75fc` (PHASE-02
corpus sweep).

**Lines of attack.** SL-163 adds a `doctrine check {quick|commit|gate}` proxy
verb and sweeps the shipped skill corpus off this repo's `just check` onto it
(POL-002 platform independence). The audit probes:

1. **POL-002 fidelity** — does the shipped product still load-bear on a host
   convention anywhere? (`just`/`mem_` leakage in `plugins/**`.)
2. **INV-1 (behaviour preservation)** — is VT `command` resolution byte-for-byte
   unchanged; do the three new `[verification]` keys perturb VT parse or
   coverage semantics?
3. **Design D6 fidelity** — did the sweep honour the CR-F4 two-treatment split
   (4 instruction-rewrites → `gate`; 2 worktree sites token-only, caller-control
   semantics preserved), not a blind grep-replace?
4. **CR-F1..F6 integration** — are the six external-review findings actually
   carried in code (Empty-arm, owned Noop, signal exit-forwarding, `-p/--path`)?
5. **Conformance algebra** — undeclared / undelivered paths vs `design-target`.
6. **Guard classification** — `check` as `Read` despite spawning source-mutating
   commands (the A1 tension).

**Invariants pinned.** INV-1 (VT frozen), INV-2 (`resolve_check` total, `Run`
argv non-empty by construction), POL-002 (defaults inform, never carry).

## Synthesis

**Closure story.** SL-163 lands faithfully. The `doctrine check
{quick|commit|gate}` verb realises the locked design §5 to the letter:
`resolve_check` is the pure leaf (override-present → `Run`; `[]` → `Empty(kind)`;
unset Quick → owned `Noop`; unset Commit/Gate → `Run(DEFAULT_*)`), `run_proxy`
inherits stdio with no timeout, and `exit_code` forwards `128+signo` on signal
death (CR-F5). The guard classifies `Check` as `Read` with the §5.3 rationale
inline (source mutation by a proxied command is a worker-legal source delta, not
an authored write). All six external-review findings (CR-F1..F6) and the six
internal A-points are carried in code or consciously accepted. Evidence: `just
check` green; `e2e_check_proxy` 5/5 (exit-forwarding incl. signal→143, keyed
ENOENT error, owned no-op exit-0); `e2e_no_shipped_couplings` 2/2 (plugins/**
free of `just`-coupling and bare `mem_` uids).

**POL-002 fidelity — holds.** The shipped corpus no longer load-bears on a host
convention: 4 instruction sites moved to `doctrine check gate`, the 2 worktree
illustrative sites updated token-only with caller-control semantics intact, the
dangling `mem_019ec65ecbc7` uid replaced by portable prose. The `just …` strings
survive only as informing argv defaults (`DEFAULT_COMMIT`/`DEFAULT_GATE`, named
constants per STD-001), never carried correctness. The `e2e_no_shipped_couplings`
guard makes the coupling non-regressable.

**INV-1 — holds.** `VerificationConfig` carries `#[serde(rename_all =
"kebab-case", default)]`; the three new `Option<Vec<String>>` keys are read
*only* by `resolve_check`, never by the VT `resolve` path. `command` resolution
is byte-for-byte unchanged; existing VT round-trip tests green unchanged
(behaviour-preservation gate satisfied).

**Findings (2, both minor, both verified, non-blocking).** Neither touches code
behaviour — both are canon-truthfulness drift handed to /reconcile:
- **F-1** — the undeclared deletion of `e2e_skills_dispatch_shrinkage.rs`
  (SL-085's dispatch-skill line-count guard). Substantively correct and
  user-approved; design §9 is silent on it. Record in design.md §9 at reconcile.
- **F-2** — slice-163.md §3 mis-frames all six sweep sites as phase gates,
  contradicting the locked design's D6/CR-F4 two-treatment split. Implementation
  followed the design (correct); the slice prose is stale. Align §3 at reconcile.

**Conformance noise (not findings).** The second undeclared cell —
`M .doctrine/slice/163/slice-163.toml` — is the slice's own lifecycle-status
metadata churn; expected, no action.

**Standing risks consciously accepted (carried from design, re-confirmed
post-impl).**
- **R5 / CR-F1** — claiming typed `quick`/`commit`/`gate` keys on the
  doctrine-owned `[verification]` table flips a *differently-typed* client key
  from silently-ignored to a hard parse error. Accepted/moot: no external client
  projects exist (single-repo reality) and the table is doctrine-owned. Revisit
  on first external adopter (tolerant-parse / migration note).
- **A5** — no proxy timeout; a hung child hangs the agent. Accepted: identical
  to running the command directly; the harness interrupts. The VT 300s cap stays
  VT-only. A configurable timeout is a possible future follow-up, out of scope.

No blocker. Ledger done (await=none). Coverage: SL-163 declares no `REQ`
coverage (SPEC-013/SPEC-010 are `concerns`, not covered requirements) — no
coverage reconciliation owed.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §9** (F-1): record that `tests/e2e_skills_dispatch_shrinkage.rs`
  was removed as part of this slice, with rationale (brittle SL-085 line-count
  cap fought the uid/sweep scrub; user-approved, net hazard > worth) and an
  explicit note that the SL-085 dispatch-skill shrink-guard invariant is
  consciously abandoned (no replacement guard owed — the new
  `e2e_no_shipped_couplings` guards the *coupling*, not line counts).
- **slice-163.md §3** (F-2): replace "rewrite the six just check occurrences …
  (mapped by cadence — all six are phase/close-boundary sites; D6)" with D6's
  two-treatment split: 4 instruction-rewrites (execute/close/audit/notes →
  `doctrine check gate`) + 2 worktree illustrative-example token updates that
  preserve project-provided / orchestrator-supplied caller-control semantics
  (CR-F4).

### Governance/spec (REV)
- None. No ADR, policy, standard, spec, or REQ change owed; POL-002 is satisfied,
  not amended.
