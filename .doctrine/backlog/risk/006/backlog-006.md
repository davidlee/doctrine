# RSK-006: SL-042 coverage scan/staleness perf — reverse-index + staleness-batching, revisited at the SL-044 reader, deferred (no cliff); build on real-scale cliff

SL-042's P3 R2 perf spike (design §9; EX-5) measured the two cost axes separately
and recorded **conditioned** backlog triggers. At measured scale both axes are
linear with no cliff. This risk holds the trigger conditions so they are not lost
(defer-needs-backlog).

## SL-044 revisit outcome (2026-06-12) — DEFERRED, trigger re-pointed

The Slice-B reader (the reconcile writer + close-gate, shipped as **SL-044**;
backlog item IMP-030, now closed) is the consumer this risk was waiting on. It
landed. Per SL-044 design (D-B3, LOCKED via `/consult`) the reverse `req→rec`
lookup is an **on-demand corpus scan, not a stored index** — chosen for ADR-004
outbound-only + anti-desync, perf escalation explicitly held here. The RV-004
audit lists the per-req `scan_coverage` corpus walks at close as a
**consciously-accepted standing risk** (`distinct_keys` de-dupes only the
symptom). No cliff was observed; the reverse-index / batching mitigation was
**deliberately not built**. So: still `open`, nothing measured-and-mitigated — the
trigger moves from "the Slice-B reader exists" to "the scan cliffs at real repo
scale." Sibling defect on the same `scan_coverage` path: **ISS-006**
(slug-symlink double-walk over-counts keys).

## Measured (debug build, ~10× release — debug-vs-release-scale-timing)

- **(a) scan fan-in** — corpus-walk + parse + filter, `IsStale` precomputed:
  N=50→3.33ms, 500→20.3ms, 2000→77.7ms (~0.039 ms/file, **linear, no cliff at 2000**).
- **(b) staleness resolution** — per-entry `git::commits_touching` subprocess:
  **~4.09 ms/call**, linear (one `merge-base`+`rev-list` pair/call) — the dominant cost.

## Conditioned triggers (re-pointed to real-scale cliff; see SL-044 revisit above)

- **Scan-axis cliff** ⇒ add a reverse index (requirement→entries) so the corpus
  walk isn't re-paid per query (the D-Q2 cost deferred behind this spike).
- **Staleness-axis cliff / many-entry reconcile** ⇒ batch staleness resolution —
  a single `rev-list` over the combined pathset, or memoize per anchor — instead
  of one subprocess per cell.

Promote to an active improvement only if a cliff appears at realistic repo scale
(the SL-044 reader did not surface one). Source: SL-042 `notes.md` (PHASE-03 R2
spike); SL-044 RV-004 audit.
