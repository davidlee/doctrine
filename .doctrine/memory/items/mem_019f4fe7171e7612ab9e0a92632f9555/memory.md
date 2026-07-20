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

Status (IMP-272, 2026-07-20): the CLAUDE arm now cures this automatically —
`dispatch_conclude_phase` mirrors each completed flip into the primary tree
alongside the coord flip (`src/mcp_server/dispatch.rs`), so a normal claude-arm
drive no longer hits the gate refusal and needs no hand-step. The manual cure
below still applies to (a) hand-flipped phases and (b) the codex/pi arm, whose
flip locus is unfixed (ISS-233).

Cure (manual paths): re-flip the phases to `completed` from the PRIMARY tree.
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
