# Review RV-259 — reconciliation of SL-206

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Final reconciliation audit of SL-206 (`workflow-slice-driver`, the `/drive-slice`
UNJAIL dispatch driver), self-audit, lightweight-close scope by operator decision
(token-thrift; no further ~200k-tok trial drives). Reviewed surface: the coord
tree `.dispatch/SL-206` (branch `dispatch/206`) — the slice's real 16/16 execution
state and the driver deliverable (`install/workflows/drive-slice.js` + dispatch
shell). Lines of attack:

- **Acceptance honesty** — does PHASE-16's green e2e actually hold, or was it
  waived? (EX-1/EX-6/P5, the (B) worker self-commit path.)
- **Deliverable soundness** — does the driver encode the arming contract the (B)
  path requires, and is the code gate clean?
- **Mechanical conformance** — does `slice conformance 206` corroborate the design
  target, and if the registry is empty, is that flagged rather than read as clean?

Invariants held: no faked green; every accepted gap consciously dispositioned with
rationale and a durable home (IMP-277); code gate (clippy) green before merge.

## Synthesis

SL-206 delivered the `/drive-slice` UNJAIL dispatch driver and its supporting
shell across 16 phases. The audit confirms the deliverable is sound and mergeable,
with one honestly-waived acceptance witness and two tolerated process gaps — no
blockers, no code drift.

**Closure story.** The slice's design (full-commit UNJAIL, superseding the earlier
confined-orchestrator posture) is realised: 15/16 phases delivered clean; PHASE-16,
the live end-to-end acceptance, ran three trial drives against the SL-209 scratch
rig. All three failed the (B) worker-self-commit witness for a **single structural
reason** (F-2): the Workflow `isolation:'worktree'` leaf is provisioned from the
primary-session cwd, not the coord arming dir, so the `worktree create-fork` hook's
cwd discriminator never matches → the worker forks detached/unmarked → the base
guard refuses it → `fork_tip` null. This is a mechanism gap in the driver (it never
calls `dispatch arm-spawn`), not a defect in the (B) path itself: PHASE-08's P5
probe already proved the armed leaf resolves `worker_commit` and runs the
commit-gate belts. The clean non-null `fork_tip` witness deferred to PHASE-16 was
therefore **waived** (F-1), by explicit operator decision, rather than faked — the
topology is now understood and the fix is bounded.

**Standing risks / tradeoffs consciously accepted.**
- (B) mode-B is capability-verified but not e2e-witnessed. The driver ships with the
  arming ritual as an *un-encoded operator step*. Until IMP-277 lands, a real
  multi-phase drive requires the operator to arm + park cwd + re-arm per phase by
  hand; an unarmed launch fails exactly as the three trials did. Documented, not hidden.
- The conformance registry is empty (F-3): the mechanical drift signal is
  unavailable, so assurance for the 16 phases rests on their individual dispatch
  reviews, not on a path-conformance delta. Flagged not-clean per the F-2 backstop.
- `check gate` is red on `doctrine validate` (F-4), but that is corpus-wide
  RFC-011 entity-validation noise (613 self-described-"expected" raw-label edges);
  the code gate, `cargo clippy --workspace`, is green with zero warnings.

Two acceptance-surfaced driver fixes already landed on `dispatch/206` during
PHASE-16 and are part of the delivered artefact: `b6a8dc70` (drop the top-level
`oneOf` from the worker schema) and `36bddeb7` (pin the arm). No reconcile action —
recorded here for provenance.

## Reconciliation Brief

No governance/spec REV and no per-slice design-edit is required — the design
(`design.md §5`, full-commit UNJAIL) already matches the implementation, and every
finding is dispositioned `tolerated`, not `design-wrong`. The only carried-forward
item is **work**, already captured:

### Per-slice (direct edit)
- None. `design.md` conforms to the delivered driver; the PHASE-16 waiver is
  recorded on the runtime phase sheet and here in `## Synthesis`, not a design edit.

### Governance/spec (REV)
- None.

### Carried-forward work (already captured — no reconcile write)
- **IMP-277** — encode the arming ritual into `drive-slice.js` (launch-time
  arm-spawn + cwd park; per-phase re-arm in the interior orchestrator's `hopPrompt`)
  so the verified-viable (B) recipe is mechanised rather than left as operator
  ritual. Obtaining the waived PHASE-16 green-e2e witness folds into this item.

### Conformance registry (tolerated, no bootstrap)
- The empty delta registry (F-3) is accepted as process-debt, not reconstructed.
  Named here so a future reader does not mistake `conformance incomplete` for clean.
