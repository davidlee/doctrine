# Notes SL-230: Memory body-write verbs and corpus-aware verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-26 · design review (RV-307 round 7) · e26f6eae00

### Produced

- RV-307 — round-7 raiser adjudication accepted F-19/F-20/F-27/F-29/F-30,
  returned F-6/F-22/F-24/F-25/F-26/F-28, and raised F-31–F-35; ledger and this
  harvest remain uncommitted for the architect.
- REV-034 — SPEC-007 + REQ-147 amendment. Proposed, applied at close.
- IMP-317 — give `validate` and `retrieve` a history-stable scope dataflow.
- IMP-318 — persist attested coverage on the verification stamp.
- QUE-175 — retrieve-side scope drift question; F-34 found its durable body
  still carries the rejected shared-constructor model.
- EVD-001, QUE-173 — from the pre-external design rounds.

### Learned

- F-31 — `git rev-list --all` is stable only over the current ref set, not over
  clones or branch deletion.
- F-32 — a textual wildcard-free prefix is not necessarily a resolvable path
  prefix; non-resolving outside pathspecs also abort the history probe.
- F-33 — a weak stamp contract and a strong scope contract cannot coexist as
  separate normative readings.
- F-34 — routing records are part of the integration sweep, not commentary
  outside the design.
- F-35 — pointer-only history must contain pointers only.

### Open

- RV-307 — F-31 and F-33 keep the design gate closed; F-32/F-34/F-35 carry the
  remaining ordered-algorithm and integration work.
- QUE-175 — correct the durable question before it can gate IMP-317 safely.
- QUE-173 — digest-based invalidation question.
- REV-034 — governance amendment reserved for close.
- R5 — masters remain outside every invalidation path.
- R7 — `validate` and `retrieve::git_facts` retain raw historical scope seams.
- R8 — the stamp does not persist the covered surface; routed as IMP-318.
