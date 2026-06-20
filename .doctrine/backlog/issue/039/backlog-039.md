# ISS-039: Dispatch claude-arm leaves boundaries.toml uncommitted on the dispatch branch — prepare-review projects 0 phase cuts

## Symptom
`dispatch sync --prepare-review` for a claude-arm drive cuts `review/<slice>` but
projects **0 phase cuts** — no `phase/<slice>-NN` refs — even when the drive
produced clean per-phase boundaries.

## Root cause
The dispatch run-ledger `boundaries.toml` is written to the **live coordination
worktree** (`.dispatch/SL-<slice>/.doctrine/dispatch/<slice>/boundaries.toml`) but
is **never committed** to the `dispatch/<slice>` branch — only `journal.toml` is
tracked there. `read_ledger` (`src/dispatch.rs`) sources the ledger from the branch
object-db (`git::read_path_at(coord_ref, …)`, deliberately, so it works stage-1 and
stage-2), so `plan_phases` reads an empty `Boundaries::default()` → no per-phase
projection.

## Evidence
Witnessed on SL-127's own dogfood drive: `boundaries.toml` carried 5 well-formed
boundaries (PHASE-01..05) in the worktree; `git ls-tree dispatch/127` shows only
`journal.toml`; `candidate status` / `sync` report `0 phase cut(s)`. Orthogonal to
SL-127's design (base-freshness); surfaced during its audit (RV-116 F-2).

## Fix direction
The claude dispatch arm (or the funnel's record step) must commit
`boundaries.toml` onto `dispatch/<slice>` alongside the journal — or `read_ledger`
must fall back to the worktree for the boundaries ledger. Confirm the
subprocess (codex/pi) arm does commit it; if so this is claude-arm-specific.

## Impact
Per-phase review granularity is lost; the cumulative `review/<slice>` bundle is
unaffected (whole delta, correct base), so it is reviewable — hence minor/tolerated
for SL-127 closure, but a real defect for dispatched-slice review ergonomics.
