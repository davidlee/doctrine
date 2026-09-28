# REV REV-068 — reconcile SL-269

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

<!-- Why this revision: what authored truth needs to change and why, the scope of
     the staged delta, and (for ADR/POL/STD/prose rows) the before/after excerpts
     the structured payload only labels. Seeded at `revision new`. -->

SL-269 changed two behaviours that three governance documents still describe
the old way. DEC-338: review writes are refused only in a dispatch worker
process, not on a fork-resolved root. DEC-337: `local` reservation reach is
clone-wide (a ref CAS in the common git dir plus a live-worktree scan), not a
per-tree `mkdir`. User-approved at reconcile, 2026-09-28.

## Reconcile narrative

- [RV-409 finding F-4] ADR-007 — D-C1 body and SL-040 clarification (fork
  refusal) superseded by an SL-269 clarification; D-C7 retitled "One writer per
  review" (any admitted tree, one writer per RV at a time, merge a best-effort
  backstop, IDE-021 for enforcement); D-C10 "refuse a worker fork" → "refused in
  a worker process".
- [RV-409 finding F-4] PRD-005 — "single-tree reach" → "clone-local reach"
  throughout; the reach definition names every working tree of one clone. The
  separate-clones limitation stands.
- [RV-409 finding F-4] SPEC-008 — `local` reach mechanism (clone-common ref CAS,
  then per-tree `mkdir`; scan union of both ref namespaces, live worktrees and
  trunk); D1 backend wording; trunk-union rationale narrowed; `reseat` lenient
  slug read, claimed destination, `--to` refusal, dangler scan scope; concerns
  updated (clone reach; post-commit partial move).
