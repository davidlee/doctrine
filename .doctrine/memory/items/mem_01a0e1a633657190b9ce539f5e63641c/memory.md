## The fact

A design run still in `reviewing` will accept `doctrine design adopt <slice>`
after `design.md` has been edited on disk. Only a LOCKED run refuses
(`AdoptionLocked`, before parsing) — that is
[[mem.pattern.reconcile.edit-design-out-of-band]], which is about the
locked-at-reconcile case, not this one.

Verified on SL-271 (2026-09-27): the design run at `reviewing` revision 38 had
`design.md` edited out of band; `design adopt 271 --dry-run` reported
`section_fingerprint_changed` for the edited sections and `unchanged` for the
rest, and the real `adopt` re-baselined the run at revision 39 and re-fingerprinted
the changed sections.

## Two lawful write paths, and how to tell them apart

- **In band** — `doctrine design apply` (one sparse mutation per call, with
  `run_uid` + `known_revision` + `submission_id`) then `doctrine design
  materialise`. This is what the prior passes on SL-271 used; the run's receipts
  name each submission (`sl271-review2-fixes-1`, …).
- **Out of band** — edit `design.md` directly, then `doctrine design adopt`.
  Cheaper for many small prose edits, because `apply` takes a whole section body.

`design show <slice> --format status` reports the stage and the pending review
pass but does **not** say which path is open, and `design tree` with no argument
may pick a different slice. Read the run state from
`.doctrine/state/slice/<NNN>/design.toml` when it matters.

## Keep section markers

`design.md` carries `<!-- doctrine:section sec-N -->` markers; adopt and
materialise map sections by them. Editing a section body is fine; removing or
renaming a marker is not.
