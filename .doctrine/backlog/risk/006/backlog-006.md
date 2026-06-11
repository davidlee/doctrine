# RSK-006: SL-042 coverage scan/staleness perf: conditioned reverse-index + staleness-batching triggers, revisit at the Slice-B reader

SL-042's P3 R2 perf spike (design §9; EX-5) measured the two cost axes separately
and recorded **conditioned** backlog triggers. Neither fires yet — at measured
scale both axes are linear with no cliff, and there is no production consumer (the
reader is the Slice-B item IMP-030). This risk holds the trigger conditions so
they are not lost (defer-needs-backlog).

## Measured (debug build, ~10× release — debug-vs-release-scale-timing)

- **(a) scan fan-in** — corpus-walk + parse + filter, `IsStale` precomputed:
  N=50→3.33ms, 500→20.3ms, 2000→77.7ms (~0.039 ms/file, **linear, no cliff at 2000**).
- **(b) staleness resolution** — per-entry `git::commits_touching` subprocess:
  **~4.09 ms/call**, linear (one `merge-base`+`rev-list` pair/call) — the dominant cost.

## Conditioned triggers (fire at the Slice-B reader, IMP-030)

- **Scan-axis cliff** ⇒ add a reverse index (requirement→entries) so the corpus
  walk isn't re-paid per query (the D-Q2 cost deferred behind this spike).
- **Staleness-axis cliff / many-entry reconcile** ⇒ batch staleness resolution —
  a single `rev-list` over the combined pathset, or memoize per anchor — instead
  of one subprocess per cell.

Revisit when IMP-030 wires the reader; promote to an active improvement only if a
cliff appears at realistic repo scale. Source: SL-042 `notes.md` (PHASE-03 R2 spike).
