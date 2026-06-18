# Review RV-084 — implementation of SL-105

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it in before raising findings. -->

## Reconciliation Outcome

### Write duty: no-op

No `## Reconciliation Brief` was authored; F-1 recorded VA-1 as pending and
requested no per-slice prose edits and no governance/spec REVs. Reconcile's
write duty was a no-op. The only gate was the VA-1 verification, executed
under reconcile per the user's direction (see notes below).

### VA-1 executed — 14/15 cleared, 1 deliberately retained

Ran `doctrine backlog after <SRC> --prune` (review/105 binary) against the
14 edge-holders. 14 in-domain item→item `after` edges cleared (targets
resolved/fixed/done/mitigated — the sound signal `--prune` was built for).
`IMP-095 → SL-095`, a valid cross-kind edge, was **deliberately retained** as
a reminder and incentive (see shortfall below).

### VA-1 procedure correction (F-1 response was wrong)

F-1's response (and the handover) inverted `--prune`'s SRC direction — they
named the *resolved targets* (IMP-008, IMP-028, …) as SRC, but `--prune`
operates on the SRC's own `after`-list, so SRC must be the edge *holder*.
They also omitted `ISS-003` (the `ISS-003 → RSK-001` override). Running the
documented procedure would have left all 15 overrides intact. The corrected
14-holder SRC set is recorded in `.doctrine/slice/105/notes.md`.

### Cross-kind shortfall — design debt, out of scope for SL-105

The actionability graph is being extended beyond backlog items to slices and
other entities. SL-105's `--prune`/`--remove` and the overrides-adapter handle
item→item `after` only; cross-kind `after` (`IMP-095 → SL-095`) is surfaced in
the `overrides:` footer but cannot be cleared (`--prune` declines on slice
status `done`; `--remove` rejects the `SL` prefix via `parse_ref`). Slice
relationships carry different semantics/ordering behaviour — extending the
verb is a design task. **Disposition:** not solved in this slice's lifecycle;
filed as **IMP-099** (triaged) for a future slice; noted in
`.doctrine/slice/105/design.md` §7 (design-significant surface) with detail in
`.doctrine/slice/105/notes.md`. Not opened as an RV-084 finding (reconcile D9:
audit owns discovery; surfaced to the user and filed under explicit direction).

### State

- 13 backlog TOMLs pruned (14 edges cleared; IMP-095's retained) — to land as
  `chore(SL-105): prune dangling item→item after edges (VA-1)`.
- IMP-099 filed, triaged, linked to SL-105.
- `design.md` §7 + §3.2 caveat added; `notes.md` reconcile section added.
- Slice ready for `/close` (land `review/105` → `main`).

Reconcile pass complete — handoff to `/close`.
