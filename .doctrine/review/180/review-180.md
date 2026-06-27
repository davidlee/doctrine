# Review RV-180 — reconciliation of SL-166

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject surface reviewed.** Not a `/dispatch`-candidate slice — workers ran
directly in worktree `.worktrees/SL-166` (no isolation). Audit reviews the fork
`slice/SL-166-corpus-loss-guards` (base `5e185148`, tip `09ee705f`,
**unlanded on edge**). Ledger driven from the primary tree (edge); evidence
gathered by reading/running in the worktree.

**What this audit probes.** Three layered mechanism-level guards (Model B,
design §7 D1) against the ISS-056 corpus-deletion hazard:
- **g1** (PHASE-04) — refuse trunk-mutating integrate on a buffer checkout.
- **g2** (PHASE-03) — base-corpus freshness at setup (`is_ancestor(corpus_tip, base)`).
- **g3** (PHASE-02) — corpus-shrink refusal at ref-advance (`merge-base(new,cur)` 3-way).

**Invariants held to.**
- INV-2 parity — posture unset ⇒ byte-identical funnel behaviour.
- ADR-001 layering — `corpus_guard` leaf must not depend on git/command tiers.
- Pure/imperative split — no git/disk in the pure validate path.
- Behaviour-preservation — existing dispatch suites stay green unchanged.
- STD-001 — refusal tokens + `.doctrine` pathspec are named constants.

**Where bodies are likely buried.** (1) PHASE-05 enablement — design §5.3/EX-1
assume a tracked-config "dedicated commit", but SL-146 moved config to
gitignored `.doctrine/doctrine.toml` the same day. (2) PHASE-05 EX-2 names a
nonexistent test target `e2e_dispatch_close`. (3) Layering-faithful
deviations from illustrative design snippets (g1 split, 3-arg
`last_corpus_commit`). (4) Conformance registry partial — PHASE-05 source-delta
binds at land, not yet recorded.

**Evidence run.** `just check` green; 67 corpus/guard tests pass; worktree
config posture-off ⇒ green e2e suite is the INV-2 parity proof; all 8 as-built
deviation claims independently confirmed at file:line; `corpus_guard.rs` imports
only `std` (layering clean); `.doctrine/doctrine.toml` confirmed untracked +
gitignored; `e2e_dispatch_close` confirmed absent.

## Synthesis

**Closure story.** SL-166 ships the three layered ISS-056 corpus-loss guards
(Model B) as designed. All three are present, correctly wired, and tested:
g1 (`guard_not_on_integration_ref` at `run_integrate`, dispatch.rs:602), g2
(`ensure_base_corpus_fresh` in the coordinate Create leg, before `worktree add`),
g3 (`corpus_clobber_check` per advance leg, `merge-base(new,cur)` 3-way,
always-on). `just check` is green; 67 corpus/guard tests pass; the `corpus_guard`
leaf imports only `std` (ADR-001 honoured). INV-2 parity holds — the worktree
runs posture-off and the full e2e suite is byte-clean green. The guard
*mechanism* matches canon; every divergence found is in the surrounding
criteria prose, not the protection.

**Two design-coherence defects, both pre-existing-cause.** F-1 (major) and F-2
(minor) are the only material findings. F-1: plan PHASE-05 EX-1 + design §5.3
specify enablement as a "dedicated commit" to `doctrine.toml`, but SL-146
(ISS-055, merged the *same day* the design was authored) moved config to a
gitignored, never-tracked `.doctrine/doctrine.toml` — the commit is impossible.
Posture was armed by an operator runtime edit instead (config validate ok); the
guard is live and correct, only the criterion is stale. F-2: PHASE-05 EX-2 names
a phantom test target `e2e_dispatch_close`; the parity it gates was actually
proven against `e2e_dispatch_sync` + `e2e_dispatch_lifecycle`. Both route to
/reconcile as per-slice direct edits — neither touches governance/spec
(ADR-012 is implemented, not amended), so no REV is needed. F-7 (minor) is a
third, lighter canon-truth edit: design §5.2 R4's "additionally" config-validate
ref check was deliberately not built (it would breach the pure/imperative split);
canon should record the omission as intentional rather than unmet.

**Faithful deviations (no change).** F-3/F-4/F-5/F-6/F-8 are as-built choices
that honour the criteria's *substance* while diverging from illustrative design
snippets — the g1 leaf+shell split, the `== deliver_to` inert leg, the
single-value g2 thread, the 3-arg `last_corpus_commit`, the call-global
allowlist. Each strengthens layering or matches the controlling EX criterion;
all disposed `aligned`. The recurring lesson: SL-166's design snippets are
illustrative, and the as-built consistently chose the layering-faithful form.

**Standing items carried to /close.**
- **Land + bind (F-9, mandatory).** The fork `slice/SL-166-corpus-loss-guards`
  (base `5e185148` → tip `09ee705f`, 03+04+05) is unlanded. At close: land from
  the **primary** tree (`worktree land --fork … --no-ff`), then — the `--no-ff`
  merge tip refuses auto-bind (F-6 non-merge-tip guard) — bind source-deltas
  manually (`slice record-delta --start 5e185148 --end 09ee705f`, or per-phase).
  Re-run `slice conformance 166` post-bind; it cannot be machine-verified until
  then. Promote edge→main only via `git fetch . edge:main`, never `checkout main`
  — g1 is now armed and a `sync --integrate` with HEAD on `main` will (correctly)
  refuse.
- **VH-1 (operator, pending).** Human eyeball of the EX-3 docs wording —
  `dispatch_config.rs:50` (R3 precondition), `dispatch.rs:78`
  (`--allow-corpus-clobber` clap help), `install/doctrine.toml.example` (posture
  block) — before close. Not an audit finding; a deferred human-verification gate.
- **PHASE-05 phase-binding.** PHASE-05 was flipped `completed` without a
  `code_start_oid` (the worktree phase-sheet regeneration reset the in_progress
  stamp). Its source delta is the `1d09b73e` docs commit; bind at land.

**Risk posture.** No blockers. The destructive hazard (ISS-056) is closed by g2
at root and defended in depth by g1/g3. Deliberately out of scope (design
Non-Goals): g4 promotion guard on raw `edge:main`, and a
raw-destructive-git pre-merge hook/policy.

## Reconciliation Brief

All remediation is **per-slice direct edit** (design.md + plan.toml for SL-166).
**No governance/spec REV** — ADR-012 and all specs are honoured as-built.

### Per-slice (direct edit)
- **plan PHASE-05 EX-1 (plan.toml:150) + design §5.3** [F-1]: replace the
  "dedicated enabling commit" enablement with the post-SL-146 env-local config
  model — `authoring-branch` is operator config in gitignored
  `.doctrine/doctrine.toml`, set at runtime, not a tracked commit. Record the
  SL-146 cause.
- **plan PHASE-05 EX-2 (plan.toml:151)** [F-2]: rename the phantom test target
  `e2e_dispatch_close` → the real INV-2 parity targets `e2e_dispatch_sync` +
  `e2e_dispatch_lifecycle`.
- **design §5.2 R4 (~:178)** [F-7]: record the config-validate-time ref check as
  **intentionally not built** (would put git/disk in the pure validate path —
  pure/imperative split); the setup-time g2 gate (VT-3) is the shipped
  protection. Drop or reframe the "additionally" so canon carries no unmet item.

### Governance/spec (REV)
- None.

### Optional (below threshold — reconciler's discretion)
- plan PHASE-03 EX-4 (plan.toml:99) [F-5]: "authoring_branch/deliver_to" → note
  g2 consumes only authoring_branch.
- plan PHASE-02 EX-4 (plan.toml:68) [F-8]: "journal row" → "journal manifest
  (call-global across legs)", matching §10 / PHASE-05 EX-3 docs.
