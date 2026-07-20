# IDE-028: Auto-push phase-status sheets to primary tree in the funnel Record beat

Follow-up to SL-190. SL-190 ships `reconcile-phases` as the **manual** primary-tree
phase-sheet writer; the primary runtime sheets (`phase-NN.toml` status) still go
stale mid-drive because the funnel Record beat only pushes the **landed registry**
(`boundaries.toml`), not the sheet status.

## Idea

Have the funnel Record beat *also* flip the primary tree's runtime phase sheet to
`completed` for the just-landed phase (a targeted write alongside the existing
`record-boundary`/`record-delta` primary-registry write — the primary-resolve seam
already exists). Result: primary `slice status` / `slice list` stay continuously
honest during a drive; `reconcile-phases` demotes to a recovery/cross-machine tool
rather than a routine pre-review step.

## Why deferred (not in SL-190)

- **Funnel-cadence change.** SL-190 scoped OUT any change to the funnel cadence
  (`slice-190.md` non-goals); this edits the Record beat.
- **IDE-027 timing gate.** Funnel-beat wrapping is deferred until RFC-005 topology
  settles (Path L/C, import-mode). Touching the Record beat now = moving target.
- Revives the "coord→primary runtime copy" shape SL-190's design consciously
  rejected as fragile — acceptable only once the beat it rides stops churning.

## Gate

Land after SL-190 (needs the composite `resolve_phase_truth` core + status-only
writer to reuse), and after RFC-005 settles the Record beat. Cheap once both hold
(the primary-write seam is already in `record-boundary`).

## Related

- SL-190 — ships the manual `reconcile-phases`; this automates the push.
- IDE-027 — the timing gate (funnel-beat wrapping deferred).
- RFC-005 — topology churn that gates touching the Record beat.
- **ISS-212** — the general bug tracker for the same root cause (coord flip not
  reaching primary); this idea is its "keep primary status continuously honest"
  enhancement flavour.
- **IMP-272** — delivered this idea's **claude-arm half** ahead of the gate:
  `dispatch_conclude_phase` now flips the primary sheet at conclude. The RFC-005
  deferral no longer applies to the claude conclude beat.

## Status (2026-07-20)

Partially overtaken. IMP-272 realised the claude-arm mechanism. What remains of
this idea:

- the **codex/pi `record-delta`** Record-beat half (primary sheet still stale on
  that arm), and
- the fuller "mirror *every* transition (not just `completed`) so primary
  `slice status`/`slice list` are continuously honest mid-drive" enhancement.

Both are cleanest as the general `set_phase_status` completion-mirror described in
ISS-212 candidate (a) — which subsumes the funnel-Record-beat framing here (the
mirror rides the shared writer, not a per-arm Record-beat wrap, so IDE-027 /
RFC-005 topology churn no longer gates it).

## Status (2026-07-20, update) — orchestrator-author + codex/pi now covered

ISS-212 shipped candidate (a): the completion mirror now lives in the single
`set_phase_status` writer (`completed`-only, guarded to the live-coord dispatch
split). That subsumes BOTH this idea's realised claude-arm half AND its still-open
codex/pi `record-delta` half — every arm that flips completion in a coord tree
now mirrors into the primary sheet through the shared writer.

Remaining open (this idea's residue): the fuller "mirror **every** transition
(not just `completed`) so primary `slice status` / `slice list` are continuously
honest mid-drive" enhancement. The `completed`-only mirror keeps the
completeness gate honest but leaves `in_progress`/`blocked`/reopen flips
primary-invisible mid-drive.
