# SL-066 — implementation notes (durable)

The REV change-axis kind (ADR-013). Harvested at audit/close from the phase sheets,
the design, and RV-029. Runtime phase sheets are disposable; this is what survives.

## Shape & seams

- **REV rides the REC eager-materialise seam verbatim** (no parallel impl): a numbered
  authored kind whose fields exceed `ScaffoldCtx` (`status`, `approval`, seeded dep/seq,
  `[[change]]` payload), so it materialises eagerly in `run_new` rather than via
  `Kind.scaffold` (the stub `rev_scaffold_unused` only satisfies the descriptor).
- **Two orthogonal axes.** `status` (work FSM: `proposed→started→done`, `abandoned`
  from any non-terminal — backlog's shape, NOT slice's 9-state) and `approval`
  (`none|requested|approved|rejected`). Lifecycle transitions are **approval-blind**
  (ADR-009 "approval is not lifecycle"): a `started` REV at `approval=none` is valid.
  Each axis is its own closed enum + `as_str` mirror + `&[&str]` known-set, kept in
  lockstep by a `*_matches_the_variants` drift canary (the backlog.rs/rec.rs pattern).
- **Edit-preserving writes** ride the shared `dep_seq::set_authored_status` seam — the
  `[relationships]` block, comments, the other axis, and unknown keys all survive (the
  file is never reserialised). Both `set_revision_status` and `set_revision_approval`
  delegate to it; the FSM gate is the *caller's* job, the seam owns only the write.

## The three corpus-walk arms (G1/G2/G3) — must co-land with the KINDS row

A new `integrity::KINDS` row is consumed by three corpus-walk tables; all three arms
landed in PHASE-02 **with** the row, or a debug-build corpus scan panics/mis-classifies
the moment a REV is minted:
- **G1** `priority::partition` — REV's OWN `KindPartition` (workable
  `["proposed","started"]`, terminal `["done","abandoned"]`) + `REV_STATUSES` const for
  the VT-2 canary. REV vocab ≠ backlog's, so it cannot ride the backlog arm; without its
  own row a `done` REV classifies `Unrecognised != Terminal` and `blocked_by`
  (channels.rs) blocks its dependent *forever* — the inverse of the IDE-010 payoff.
- **G2** `relation_graph::dep_seq_for` REV arm — mirrors the SL arm; reads
  `revision-NNN.toml` directly so REV-as-source `needs`/`after` reach the blocker/`next`
  view. The scaffold seeds an empty `[relationships]` block so the read is total.
- **G3** `relation_graph::outbound_for` REV arm — routes to `revision::relation_edges`
  *before* the `debug_assert!(false)` fallthrough. The accessor was an empty stub in
  PHASE-02, filled in PHASE-03 — but the arm had to exist in PHASE-02.

## `[[change]]` payload (PHASE-03)

- **The rows ARE the edges** (members.toml precedent): each `[[change]]` row projects to
  one `Revises` edge. `revises` is `TypedVerbOnly` — authored only by `revision change
  add`, never `doctrine link`; the RELATION_RULES row exists for target validation
  (`{SPEC,PRD,REQ,ADR,POL,STD}`, off-target refused) + inbound naming. Inbound surfaces
  on `inspect ADR-X`/`inspect REQ-N` (every touching REV, uniform), NOT on `show`
  (ADR-004 §3 reserves inbound completeness to the scan-backed surface).
- **Two row shapes, one table** (F3 — creation ops can't key on an FK that doesn't exist
  yet): existing-target ops (`modify|retire|move|status`) key on a live FK; creation ops
  (`introduce|create`) carry a **frozen** `new_label` (REQUIRED, E4 — else membership
  churn between draft and apply silently changes what lands) + a live `member_of` SPEC.
- `primary` is a **display/headline hint only** (F1): at-most-one, optional, nothing
  functional keys on it.
- **OQ-1 dedup**: a second row of the same `(action, target)` for an existing-target op
  is refused — a change is named once.

## Apply path (PHASE-05)

- **v1 auto-lands `status` rows ONLY** — they ride the engine-callable
  `requirement::set_status` (defined `requirement.rs:339`; `spec.rs:897` is just the
  existing call site → no refactor, no ADR-001 violation). `introduce`/`create`/`modify`/
  `move`/prose rows are **surfaced-for-manual** (listed, landed by operator hand-edit):
  `spec req add`/`spec new` are non-transactional, so auto-applying risks orphaned
  half-writes. → carried as **IMP-074** (transactional creation-apply).
- **Approval checkpoint**: apply REFUSES unless `approval = approved`. Invoker-blind —
  records THAT an approval happened, not WHO (ADR-009; a solo dev self-approves).
- **All-or-nothing from-guard**: a pre-flight sweep reads the CURRENT `ReqStatus` for
  every status row; if any `current != row.from` (or the target is missing) the WHOLE
  apply aborts, surfaces the full stale set, and writes nothing. Drift-surface posture
  (never silently clobbers an intervening reconcile move). The `from` snapshot is
  captured at `change add` time, not apply time — the correct semantic for drift.
- **`done` never lies** (M1): a status-only REV → `done` (dependents unblock); a REV
  also carrying surfaced-for-manual rows stays `started` post-apply until the operator
  completes them by hand.
- **REC schema untouched**: `compose_apply_rec` mirrors reconcile's status-REC but is
  standalone — `RecMove::Revise`, `owning_slice = None` (non-slice-close change), one
  `[[status_delta]]`, **empty** `evidence_ref` (apply rests on the approved REV, not a
  coverage scan). One commit carries the N status edits + N RecDocs (grain differs from
  SL-044's one-act-one-commit deliberately — N self-describing acts, NF-003).

## dep/seq membership (PHASE-04)

- `is_work_like` widened to `{ slice } ∪ { 5 backlog kinds } ∪ { revision }`. REV is
  admitted as **both** dep/seq source and target (a slice may `needs REV-N`; a REV may
  `needs` a spike — the IDE-010 payoff). Governance docs (spec/ADR/POL/STD) stay
  EXCLUDED as dep/seq targets — depending on governance routes THROUGH a Revision, never
  the evergreen doc (the SL-060 invariant). This is the one widen-later guard.

## Reconciliation (RV-029) — disposition of the external code review

10 findings, all terminal, no blocker. Six actionable quality items → **IMP-073**
(test-harness DRY, module decomposition, unit-test the row-build validation,
`settle_disposition` `_=>` trapdoor hardening, magic-`0` placeholder, TOCTOU doc).
Tolerated/aligned: `allocated` is the design-intended operator-hand-fill anchor
(design.md:228 — automated producer is IMP-074); whole-file parse rides the REC
precedent; dup-slug is the consistent all-kinds posture; the "branch lags" finding is a
diff-base artifact (merge-tree onto current main is 0-conflict).

## Carried / deferred

- **IMP-073** — SL-066 REV quality hardening (the six actionable RV-029 findings).
- **IMP-074** — transactional creation-apply (OQ-3 / B2); also `allocated`'s producer.
