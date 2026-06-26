# REV REV-012 — reconcile SL-157

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Reconcile of SL-157 (RV-166 finding F-2). SL-157 PHASE-01 strips the integrate
not-checked-out leg's speculative post-CAS re-probe/resync (`advance_pure_ref` is
now pure CAS-and-done; `resync_worktree_hard` + `Disposition::RacedDesync`
retired). SPEC-022's advance-leg prose still describes the deleted mechanism, so
it must be amended to match the implementation. The D4 CAS contract is unchanged
(every advance still a 3-arg CAS, non-FF still refused) → **no ADR-012 Revision**;
this is the sole durable-governance touch.

### Change row

- **modify SPEC-022** (`spec-022.md`) — strike the post-CAS-resync parenthetical
  from the advance-leg paragraph.

  **Before:**
  > a not-checked-out target advances by pure `update_ref_cas` (with a post-CAS
  > re-probe that resyncs a newly-checked-out ref); a checked-out target advances
  > by `merge --ff-only` …

  **After:**
  > a not-checked-out target advances by pure `update_ref_cas` (CAS-and-done — the
  > delivery ref is never checked out, SL-157); a checked-out target advances by
  > `merge --ff-only` …

  (Final wording to be confirmed against the live line at landing.)

### Sequencing — deferred apply (design §5)

Design §5 mandates the spec strike land **after the code lands** (spec must never
lead code). The SL-157 code is currently fork-only (`da243b3d` on
`sl-157-phase-01`, not on `edge`/`main`). Per the reconcile-sequencing decision
(2026-06-26), this REV is authored + **approved but not applied** in the reconcile
pass; `revision apply` + the manual prose landing + `revision status done` are
deferred to **/close**, co-landed with the fork code. The REV hands to close as
`started`.
