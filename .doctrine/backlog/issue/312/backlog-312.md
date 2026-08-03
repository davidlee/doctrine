# ISS-312: backlog list drops knowledge-record needs as dangling

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

The knowledge workflow explicitly supports gating work on an unsettled question:
the dependent work item authors a `needs` edge to a `QUE-NNN` record, and the
question's terminal status later unblocks it.

On 2026-08-03:

1. `doctrine backlog needs CHR-053 QUE-205` succeeded and validated both refs.
2. `doctrine backlog show CHR-053` rendered `needs: QUE-205`.
3. `doctrine knowledge show QUE-205` rendered the record, first as `open` and
   later as `answered`.
4. `doctrine backlog list --tag cluster:capsule-spike` nevertheless printed
   `CHR-053 → QUE-205 dropped (dangling: QUE-205 absent)` in both states.

The authored edge and both entities remain intact; the defect is in the backlog
ordering/list reader's target catalogue or terminal-state handling.

## Expected

- An open knowledge question referenced by `needs` is recognized as an existing
  hard blocker.
- An answered or otherwise terminal question is recognized as settled and no
  longer blocks ordering.
- Neither state is reported as a dangling absent ref.

## Impact

The documented work→question gating pattern becomes misleading: capture and
entity reads accept the edge, but the primary backlog survey drops it as absent.
An orchestrator can therefore miss a live epistemic blocker or mistake a valid
settled dependency for corpus corruption.

IMP-033 delivered broader cross-kind dependency capture for work-like entities;
this issue is the narrower knowledge-record target path.
