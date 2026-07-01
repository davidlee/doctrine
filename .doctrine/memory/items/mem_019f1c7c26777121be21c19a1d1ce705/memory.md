# Completing a phase from the main tree clobbers worktree source-delta boundaries

**Symptom.** `slice conformance <ID>` reports every declared `design-target`
selector as `undelivered` (0 conformant) even though the code was clearly
delivered. Cause: the phase's `boundaries.toml` row has `code_start_oid ==
code_end_oid` (a degenerate, empty-diff range).

**Root cause.** For a slice whose phases were executed and committed in a
`/dispatch` **worktree**, the real source-delta boundaries are recorded against
the worktree branch commits. Flipping a phase to `completed` **from the main
tree** re-runs the automatic solo phase-binding, which stamps a boundary at the
main-tree HEAD for both start and end (the F-2 "a completed phase must carry a
row" backstop). This **UPSERTs by phase**, overwriting the correct range.

**Do not** flip phase status from the main tree for a worktree-delivered slice
whose boundaries are already recorded. During `/audit`, if conformance reads
`incomplete` only because phase status lags, prefer leaving status as-is or
re-recording the deltas — not a naive `completed` flip.

**Recovery.** Re-record the true ranges with the sanctioned escape hatch:
`doctrine slice record-delta <ID> PHASE-NN --start <oid> --end <oid>` (UPSERT by
phase; guards `start` ancestor-of `end`, non-merge `end`). Registry is
gitignored runtime state under `.doctrine/state/slice/<NNN>/boundaries.toml`.

Seen during SL-184 audit. See [[signpost.doctrine.audit]].
