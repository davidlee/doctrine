# IMP-293: Ratchet-red worker halt needs an orchestrator-handoff signal

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced at SL-220 PHASE-06 dispatch. The `worker_commit` gate correctly
refused `commit-gate-red` when the architecture-layering tangle ratchet
(`tests::architecture_layering_gate`) failed on design-mandated coupling
growth — but the ratchet baseline lives in `.doctrine/adr/001/layering.toml`,
a worker-forbidden zone. The worker cannot fix the red and cannot retry
around the refusal, so a legitimate, green-everywhere-else delta is
un-committable by the worker alone. The bump is structurally an
orchestrator-owned governance edit, yet nothing in the worker contract or
refusal payload distinguishes "ratchet-only red — hand to orchestrator" from
"worker-delta defect — fix your code".

Resolution at SL-220: orchestrator adjudicated + bumped the baseline
(86→99, commit 7661b3fd on `dispatch/220`), then a fresh worker replayed
the delta verbatim onto the bumped base — primary path preserved, but at
the cost of one extra spawn and a full hand-back round trip.

Candidate shapes:
- `check commit` (or the `worker_commit` refusal payload) classifies which
  failing gates are forbidden-zone-remediable and says so in the refusal
  (`commit-gate-red{orchestrator-handoff: tangle-baseline}`).
- The layering gate prints the full per-tier baseline/actual table on
  failure (worker case-note: only violating tiers print today), making the
  orchestrator bump mechanical.
- Dispatch skill documents the ratchet-red → bump-on-coord → replay
  pattern so orchestrators don't rediscover it.

See RFC-011 case-notes entries `[dispatch-worker; SL-220 PHASE-06]` and
follow-up for the token-cost accounting.
