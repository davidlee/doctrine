# Review RV-234 — reconciliation of SL-190

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed (F-2 record).** Dispatched slice. Evidence refs (`review/190`,
`phase/190-01..06`) are immutable (R2). Audit ran against the **candidate
interaction branch** `candidate/190/review-001`, tip `0eca671a` — the no-ff 3-way
merge of `review/190` onto `main`, conflict-resolved and gate-normalized during
this audit (worktree
`.doctrine/state/dispatch/candidate/cand-190-review-001`). Admit via
`dispatch candidate admit --slice 190 --id cand-190-review-001 --review RV-234`.

**Lines of attack.**

1. **Path-conformance algebra** — `slice conformance 190`: undeclared
   (guard.rs, reconcile.rs, 4 e2e tests), undelivered (cli.rs). Each a lead, not a
   verdict: scope creep, a stale design attribution, or an incidental ripple?
2. **Design ⇄ implementation fidelity** — did the RV-214-hardened decisions land as
   written? Composite landed-truth (refs⊕cache⊕coord), the total truth table, the
   status-only reconcile writer (no registry mutation, F-3), refuse-when-live (F-5),
   the gc-oracle extraction-vs-generalization split (F-6), shared selector predicate
   home in conformance.rs (F-7).
3. **ADR-001 pure/imperative split** — `resolve_phase_truth`, `classify_worktree`,
   `diagnose_selector` must be pure (no clock/git/disk); impurity in the shell.
4. **POL-002 platform independence** — no host-build coupling snuck in
   (binary-freshness / provisioning were explicitly excluded).
5. **Behaviour-preservation gate** — gc suite green **unchanged** after the oracle
   lift (F-6.1); the generalized non-`HEAD`-target path carries its own new tests
   (F-6.2).
6. **Open questions** — OQ-1/OQ-2/OQ-3 resolved in impl, none leaking.
7. **Integration integrity** — the base-staleness conflict and any fmt drift the
   funnel let through.

**Evidence gathered.** Candidate builds clean; `doctrine check gate` EXIT 0 (clippy
zero-warning, full suite green); `slice verify-vt 190` → 11 VT PASS + 1
UNATTRIBUTABLE (PHASE-04 VT-2, correct-by-design — it asserts the gc e2e stays
*unmodified*, the F-6.1 behaviour-preservation gate); the four SL-190 e2e suites
(phase_status 3, reconcile_phases 4, selector_doctor 4, worktree_list 5) all pass.

## Synthesis

**Closure story.** SL-190 delivers IDE-027's "now" half — four orchestrator-facing
state-visibility verbs (cross-tree phase-status query + reconcile, worktree
inventory, selector doctor) — and the implementation is faithful to the
RV-214-hardened design. Every load-bearing decision landed as written and is
proven by tests, not implied:

- **Composite landed-truth** (durable `phase/*` refs ⊕ derived registry cache ⊕
  live coord runtime) and the **total truth table** are realized by the pure
  `resolve_phase_truth` returning `Vec<(PhaseId, PhaseTruth)>` + `Divergence` —
  per-phase, not rollup-aggregated (F-2 of RV-214). The catch-all `UNKNOWN`
  totality and CONFLICT (rework) arms are present.
- **Status-only reconcile writer** (F-3): the writer lives in state.rs, edit-preserving,
  and the e2e proves zero registry mutation — the read-then-clobber cycle designed
  against is absent.
- **Refuse-when-live** (F-5) and **degrade-for-non-dispatch** (F-6-prior) both hold.
- **gc-oracle split** (F-6): the extraction is behaviour-preserving — the gc suite is
  green **unchanged** (VT PHASE-04 VT-2 is UNATTRIBUTABLE precisely because the
  slice deliberately leaves tests/e2e_worktree_gc.rs untouched, which is the gate
  passing, not failing); the generalized non-`HEAD`-target path carries its own new
  cases.
- **Shared selector predicate** (F-7): `diagnose_selector` is homed in conformance.rs,
  reusing the one `glob::Pattern`/`glob_matches` machinery — no parallel matcher. The
  integration merge (F-2 below) proved this seam empirically: SL-180's and SL-190's
  conformance.rs additions are disjoint and compose.
- **ADR-001 purity** holds across all three cores (resolve_phase_truth,
  classify_worktree, diagnose_selector — std + glob leaf only). **POL-002** clean —
  no binary-freshness or provisioning coupling crept in. **OQ-1/2/3** all resolved in
  impl (reconcile-phases name; conformance.rs predicate home, SL-190 lands first;
  --across-trees opt-in).

**Standing risks / consciously accepted.** None material. Findings were four, all
minor/nit, none gating:
- F-1 (design §Layering table mis-attributes routing to cli.rs) — a design-artifact
  truth-lag, remediated by /reconcile (brief below). The implementation is correct;
  the table is stale.
- F-2 / F-3 (base-staleness merge conflict + rustfmt drift in the impl bundle) —
  integration-seam hygiene, resolved in the candidate (tip 0eca671a); no SL-190 code
  touched. Both trace to dispatch-funnel process, not this slice's authorship.
- F-4 (tests undeclared) — conformance-cell noise, aligned.

**The one novel surface this audit authored** is the conformance.rs union merge —
code no prior review saw. It is a mechanical union of two disjoint symbol sets that
both consume the shared `compute`/`glob_matches` core; it compiles, clippy is clean,
and the full suite (both slices' conformance tests) is green. Confidence: high.

**Cross-machine boundary** (design-documented limit) stands honestly: reconcile
recovers *landed* truth from fetched `phase/*` refs and marks never-committed
in-flight phases `unknown` — the narrower, honest precondition RV-214 F-1 installed.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §"Layering & code impact" table** (F-1): the routing/impact table is
  stale. Drop the `src/commands/cli.rs | route new subcommands` row — cli.rs was
  never needed (the new verbs are variants of existing `SliceCommand` /
  `WorktreeCommand` / `SelectorCommand` enums; clap routes them without a cli.rs
  edit, confirmed by conformance reporting cli.rs *undelivered*). Add two rows for
  the real (conformance-undeclared, legitimate) touches:
  - `src/commands/guard.rs | worker-mode read/write classification for reconcile-phases (Write), selector doctor (Read), worktree list (Read)`
  - `src/reconcile.rs | call-site ripple: run_status gained the --across-trees/--assert bool params`

### Governance/spec (REV)
- **None.** No ADR, policy, standard, spec, or requirement finding. All four findings
  are per-slice or resolved-in-candidate. No REV required.
