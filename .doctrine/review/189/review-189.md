# Review RV-189 — reconciliation of SL-172

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject reviewed.** Dispatched slice (SL-172, `/dispatch`). `review/172`
(7aaaff6d) and `phase/*` are immutable evidence refs (R2). Audit ran against the
**candidate interaction branch** `candidate/172/review-001`
(cand-172-review-001, base `refs/heads/main`, merge of `review/172`), per the
dispatch-candidate flow. Diff vs main: `src/priority/{graph,config,surface}.rs` +
`src/commands/config.rs` (+408/−67), no test or `.doctrine/` edits.

**What this audit probes.**

1. **Cost-model correctness** — `est_cost` implements the design's skew + bare
   anchor (ADR-015 §1 cost term): `has_estimate → lower + β(upper−lower)`,
   `bare → max_upper(corpus) + margin`, empty-corpus → `1.0` fallback. Invariants
   INV-1 (β=0.5 ≡ legacy midpoint), INV-2 (anchor dominance over non-terminal
   estimated items), INV-3 (`est_cost ≥ EPSILON`, no div-by-zero).
2. **Config surface parity** — `[priority.estimate] {skew, margin}` clamps
   (skew∈[0,1], margin≥0, defaults 0.65/1.0) and `config show/set/get/unset`
   reach both keys.
3. **Design↔impl conformance** — every divergence between `design.md` and the
   landed code dispositioned; the NF-001 facet-naming tripwire honoured
   (graph.rs cost fn must not name facet types).
4. **Mechanical path-conformance** — `slice conformance` algebra: undeclared /
   undelivered cells run to ground.
5. **Owed governance** — the slice declares REV-routed amendments (ADR-015
   §1+§2+§4; SPEC-020 REQ-310/FR-011 v1-aggregation deferral lift); audit records
   them for the reconciliation brief, does not write them.

**Invariants held.** Behaviour-preservation gate (existing suites green
unchanged); pure/imperative split (no corpus read inside the pure scoring fn —
`max_upper` threaded as a `CostCtx` input); STD-001 (no magic strings); the
storage rule.

**Evidence run.** Full suite on the candidate: **2755 + all integration suites
pass, 0 failed, 1 ignored**; both declared e2e goldens
(`e2e_priority_golden`, `e2e_priority_cross_kind`) **green unchanged**; clippy
`--workspace` **zero warnings**. (Candidate worktree required `web/map/dist/`
— gitignored built web assets — copied in to compile; an env/provisioning
artifact, not a slice defect.)

## Synthesis

**Closure story.** SL-172 lands the ADR-015 cost-model fix cleanly: `est_cost`
replaces the inline midpoint divisor with skew (`lower + β(upper−lower)`, default
β=0.65) and a data-driven bare anchor (`max_upper(corpus) + margin`, default
margin=1), with an empty-corpus `1.0` fallback that makes the ISS-057 inversion
structurally impossible. The three named invariants (INV-1 legacy equivalence at
β=0.5, INV-2 anchor dominance over non-terminal estimated items, INV-3 EPSILON
floor) are each covered by unit tests and verified green. The operator surface
reaches parity: `[priority.estimate] {skew, margin}` with clamps and
`config show/set/get/unset` coverage. Full suite (2755 + integration, 0 failed)
and `clippy --workspace` (zero warnings) pass on the candidate bundle — the
behaviour-preservation gate holds.

**The one real design↔impl gap (F-1).** `est_cost`'s signature took plain bounds
`Option<(f64,f64)>` rather than design §5.2's `Option<&EstimateFacet>`. This was
not drift — it is the *correct* resolution of a conflict the design author missed:
naming `EstimateFacet` inside graph.rs's cost fn trips the NF-001 tripwire (facets
route through the local `EntityFacets` struct; graph.rs never names facet types).
The implementation honours NF-001; the design prose is stale. Reconcile amends the
design to match the code — code is authoritative here.

**Standing risks / consciously accepted.**
- *F-2 — golden over-declaration.* design.md predicted two e2e goldens would
  deliberately recompute; they didn't (green unchanged). Harmless to behaviour
  (tests pass), but canon over-states a change. Reconcile corrects the prose so
  design tells the truth — a cheap accuracy fix, not a defect.
- *F-3 — canon lags shipped behaviour until the REV lands.* ADR-015 still
  describes the midpoint model and SPEC-020 still carries the v1 aggregation
  deferral. This is *expected* between audit and reconcile — the governance write
  is reconcile's REV surface, deliberately not blocking the audit→reconcile move.
  The risk is only realised if reconcile is skipped; the brief below makes the
  owed edits explicit so it cannot be silently dropped.

**Out of audit scope (recorded, not findings against SL-172).**
- *Dispatch-harness incidentals* (RFC-011 case-notes): pi-RPC spawn now kills on
  `agent_end` instead of idling out the timeout; worktree fork refuses inside a
  worker-stamped CWD (must run from orchestrator root); `--lib` dropped (binary
  crate). Already captured in `.doctrine/rfc/011/case-notes.md`.
- *Candidate-worktree provisioning gap*: a fresh candidate worktree lacks the
  gitignored `web/map/dist/` build assets, so the bin fails to compile until they
  are copied in. Audit-tooling friction, not slice behaviour. Captured as a
  backlog chore + RFC-011 case-note.

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md §5.2 (F-1):** correct the `est_cost` signature to the landed form
  `fn est_cost(bounds: Option<(f64,f64)>, ctx: CostCtx, ec: &config::EstimateCost)
  -> f64`, and add a one-line note that the bounds-tuple param (not
  `&EstimateFacet`) exists to honour NF-001 — graph.rs's cost fn must not name
  facet types; the caller destructures bounds from `EntityFacets` before the call.
- **design.md §307-308 (F-2):** demote `tests/e2e_priority_golden.rs` and
  `tests/e2e_priority_cross_kind.rs` from "recompute (deliberate, reviewed)" to
  *declared-but-verify-unchanged* — audit ran both on the impl bundle and they are
  green without edit. Optionally drop them from the design-target selector set so
  conformance stops flagging them undelivered.

### Governance/spec (REV)

- **ADR-015 §1+§2+§4 (F-3):** REV modify — replace the `est_cost = midpoint`
  cost term with the skew + bare-anchor model
  (`has_estimate → lower + β(upper−lower)`; `bare → max_upper(corpus) + margin`;
  empty-corpus → `1.0`), including β/margin defaults (0.65 / 1) and the
  `[priority.estimate]` config knobs.
- **SPEC-020 REQ-310 / FR-011 (F-3):** REV — lift the v1 aggregation-deferral
  language now that the corpus `max_upper` aggregation ships; reconcile REQ-310
  status to reflect the delivered aggregation.
