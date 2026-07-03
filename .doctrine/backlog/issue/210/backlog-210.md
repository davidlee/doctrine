# ISS-210: Exposed role/worker starter self-replaces, re-suppressing Framework worker contract on new installs

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Deferred from SL-191 PHASE-07 (user call, 2026-07-04): fix this repo's overlay
in-place there; defer the Framework projection-default root cause here.

## Problem

`role/worker` is in the **expose** set (`install/manifest.toml [hymns].expose`).
So `doctrine install --project-starters` projects an editable starter to
`.doctrine/hymns/role/worker.md` carrying a sidecar `replaces = "role/worker"`
(the SL-193 self-replace projection). That starter is thin/generic and FULLY
SUPPRESSES the Framework `role/worker` slot — including the PHASE-02 universal
worker-contract enrichment (negative contract, hermetic goldens, path-scoping,
function-home, verify-as-you-go).

Consequence: every NEW `doctrine install` re-creates the suppressing twin, so a
fresh project's `prompt resolve --role worker` gets only the thin starter and
never sees the enriched Framework contract. SL-191 PHASE-07 fixes only THIS
repo's hand-managed overlay; the projection default reproduces the ISS-206
suppression for all new installs.

## The decision to make

Reconcile the expose/self-replace projection default so the enriched Framework
contract composes for new projects too. Candidate directions (needs a design):

- **Seal `role/worker`** (move expose→seal): Framework wins, no user twin;
  per-project tailoring only additive via the `project` band. Cleanest
  composition, but removes wholesale user override of the worker role and
  partly reverses SL-193's deliberate expose-with-self-replace.
- **Expose but project an EMPTY / compose-oriented starter** (no self-replace,
  or an additive `project`-band starter instead of a replacing `role/worker`
  twin): keeps customisation, but a non-replacing same-slot twin double-emits
  (ISS-206) unless it lands in a different (additive) slot.
- **Split**: keep `role/worker` framework-canonical (seal), expose a NEW
  additive `project/worker-habits` starter for per-project customisation.

Touches `install/manifest.toml` (Framework-wide) and the projection logic in
`src/install.rs` (`project_starters`, `embedded_expose_set`/`embedded_seal_set`).
Likely its own slice (governance-adjacent: a projection-contract change).

Relates to SL-191 (PHASE-02 enrichment, PHASE-07 in-repo reconciliation),
SL-193 (self-replace projection), ISS-206 (role/worker doubling).
