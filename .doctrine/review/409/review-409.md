# Review RV-409 — reconciliation of SL-269

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Surface reviewed: branch `capsule/SL-269/c26` at `9dbe1b8ae` (adopted capsule
generation 26), range `5cd5eec8e..HEAD`, in worktree `.worktrees/SL-269-c26`.
This ledger was itself opened in that capsule worktree with the slice's own
binary: a live check of the review-admission change (DEC-338).

Lines of attack:
- Admission (DEC-338): every write verb routes through `resolve_review_root`,
  refused only when `DOCTRINE_WORKER` is set; read verbs unaffected; `new`
  refuses before allocating.
- Clone-wide reservation (DEC-337): `local` claims are a ref compare-and-swap
  (CAS) in the common git dir; the scan covers sibling worktrees.
- `reseat`: lenient slug read, claimed destination, dangler worklist.
- Conformance: design-target selectors against the paths actually touched.
- Guidance and governance text agree with the new rules.

Evidence: `doctrine check gate` green; `slice verify-vt` 25/25 PASS;
`slice conformance --against 5cd5eec8e..capsule/SL-269/c26` (47 undeclared,
2 undelivered, 25 conformant).

## Synthesis

The implementation matches the locked design on every behavioural axis. The
gate is green and all 25 test-verified (VT) criteria pass. The two decisions
hold in the code:

- **DEC-338 (review admission).** `resolve_review_root` finds the root, then
  `admit_review(worker_process())`. All write verbs (`new`, the turn verbs,
  `status`, `prime`, `unlock`) route through it; `show`/`list` do not. The MCP
  path goes through the same function, so the CLI `worker_guard` bypass is
  closed (`mem.pattern.review.mcp-bypasses-worker-guard`). This ledger was
  opened, primed and driven in an adopted capsule worktree with the slice's
  binary, which the pre-slice binary refuses.
- **DEC-337 (clone-wide local reservation).** RV-409 was claimed as
  `refs/doctrine/reservation-local/RV/409` in the common git dir, and the scan
  skipped edge's RV-408.

All four findings are documentary: the design and governance lag the code.
None is a code defect.

Standing risks:
- **Transition window.** Until this lands and the edge binary is rebuilt, a
  pre-slice binary allocates by per-tree `mkdir` and ignores
  `reservation-local` refs. An RV minted on edge before the merge could
  collide with RV-409. Detect with the merge; repair with `reseat`.
- **Two writers on one RV** (design sec-8): accepted; enforcement is IDE-021.
- **DEC-337's accepted gap**: ids minted before the slice in trees that are no
  longer live are invisible to the scan.

## Reconciliation Brief

### Per-slice (direct edit)
- F-1: selector registry — `doctrine slice selector rm` the two
  `.agents/skills/{audit,inquisition}/SKILL.md` targets and `add`
  `plugins/doctrine/skills/{audit,code-review,inquisition}/SKILL.md`. Mirror in
  design.md sec-5 (guidance table rows; drop the "generated copies
  `plugins/doctrine/skills/**`" claim) and sec-6.
- F-2: `doctrine slice selector add` for `src/kinds/mod.rs`,
  `src/test_support.rs`, `tests/common/mod.rs`. Mirror in design.md sec-6
  (the `kinds` row: review `state_dir` → `None`; the `LinkedTrees` fixture row).
- F-3: record `fulfils` edges from SL-269 to ISS-279, ISS-277, ISS-483,
  ISS-494, IMP-240, IMP-190, ISS-496 and ISS-292 (partial: faults 1, 3, 4),
  per design sec-5 Backlog.

### Governance/spec (REV)
- F-4: one REV per design sec-5 "Governance":
  - ADR-007 D-C1 (lines 40-42): refuse only in a worker process, not a
    fork-resolved root.
  - ADR-007 D-C7: one writer per RV at a time in any admitted tree; git merge
    is a best-effort backstop.
  - ADR-007 D-C10 (line 159): "refuse a worker fork" → "refuse in a worker
    process".
  - PRD-005: `local` reach means this clone; trees of one clone no longer
    collide; separate clones still can.
  - SPEC-008 (lines 73, 143, 155): local = clone-common ref CAS plus per-tree
    `mkdir`; scan union; `reseat` does a lenient slug read, claims its
    destination, and refuses `--to` onto a held id.

## Reconciliation Outcome

### Direct edits applied
- F-1: selectors — removed `.agents/skills/{audit,inquisition}/SKILL.md`; added
  `plugins/doctrine/skills/{audit,code-review,inquisition}/SKILL.md` as
  design-target. design.md sec-5 (skill-source sentence, guidance table incl. a
  `code-review` row) and sec-6 (guidance row) mirror it.
- F-2: selectors — added `src/kinds/mod.rs`, `src/test_support.rs`,
  `tests/common/mod.rs` as design-target; design.md sec-6 gains a `kinds` row
  and a `LinkedTrees` fixture row. Conformance now reports 0 undelivered and no
  undeclared source, test or plugin path.
- F-3: premise mostly false. Seven of eight `fulfils` edges already existed;
  `doctrine show SL-269` does not render outbound `fulfils` edges (ISS-498).
  Added the one missing edge, SL-269 fulfils ISS-494. IMP-240 stays `full`, not
  `partial` as the brief said: its defect 1 was fixed by ISS-484 and defect 2 by
  this slice.

### REVs completed
- REV-068 (`reconcile-sl-269`): done — ADR-007 D-C1/D-C7/D-C10 plus the D-C7
  verification line; PRD-005 "single-tree" → "clone-local" reach; SPEC-008
  local-reach mechanism, D1, trunk-union rationale, `reseat` section and
  concerns (covers F-4). Rationale in revision-068.md.

### Withdrawn / tolerated
- None.

Reconcile pass complete — handoff to /close.
