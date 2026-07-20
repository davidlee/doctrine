# Dispatch phase statuses must be completed in the primary tree before prepare-review

prepare-review's completeness gate reads completed-phase ids from the PRIMARY tree's runtime state; funnel status flips made only in the coord tree leave it refusing with "recorded row … not a completed phase".

Mechanics: `dispatch sync --prepare-review` calls
`registry_completeness(&primary, &primary, slice)` (src/dispatch.rs:1918) —
recorded rows come from the shared primary-resolved
`.doctrine/state/slice/NNN/boundaries.toml`, but the completed set comes from
the primary tree's gitignored, per-worktree phase sheets
(`completed_phase_ids`, src/state.rs). A dispatch drive whose
`slice phase --status` flips all ran in the coord tree therefore fails the
gate even though `dispatch status` (coord-side) shows every phase completed.

Status (ISS-212, 2026-07-20): now cured GENERALLY. The completion mirror lives
in the single writer `set_phase_status` (`src/state.rs`), not per call site — so
EVERY path that flips a phase `completed` in a coord tree (claude-arm
`dispatch_conclude_phase`, orchestrator-author raw `slice phase --status`, and
the codex/pi `record-delta` remainder) auto-mirrors into the primary sheet. The
mirror fires only on the dispatch split: `primary != project_root` AND a live
`dispatch/<slice>` coordination worktree exists (a solo `/worktree` fork is
narrowed out so it never records a bogus primary row). Degrading — a mirror fault
warns, never fails the flip. No hand-step is needed on any arm anymore. (The
earlier IMP-272 patch mirrored only in `dispatch_conclude_phase`; ISS-212
relocated it down into the writer and removed that inline block.)

Cure (fallback only — needed just for a stale sheet from a pre-ISS-212 drive or
a mirror that degraded): re-flip the phases to `completed` from the PRIMARY tree.
This is safe while the coordination worktree is live — the solo phase-binding
capture self-skips when a live worktree holds `dispatch/<slice>`
(`capture_phase_boundary` arm guard, src/state.rs:542), so the registry rows
recorded by the funnel are never clobbered. Verify row count + oids after.

Sibling footgun: dispatch's ref-writing verbs (import, conclude,
prepare-review's journal commits) advance the checked-out coord ref via the
object db WITHOUT touching the working tree — follow each with
`git restore --source=HEAD --staged --worktree -- <paths>` or the tree shows
phantom staged deletions. Surfaced during the SL-213 drive (RFC-011 case
notes SL-213-drive-4/5/6).
