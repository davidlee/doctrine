# IMP-346: Non-dispatchable phases in the funnel

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine dispatch next` has no concept of a phase that *cannot* be dispatched.
It prescribes `spawn` for every phase without a funnel row, including phases
whose work is authored `.doctrine/` state — which a worker never writes
(ADR-006 sole-writer; `mem_019eb5eb158f7a70a200aad52a873712`).

The orchestrator's only recourse is to knowingly not act on a prescription,
which collides with the funnel's own discipline: *ask `next`, do the one thing
it says; a refusal is the recovery procedure, never re-drive around one.*

## Where it bit

SL-233 has three coordinator-only phases — PHASE-01 (governance descent),
PHASE-02 (pure run model, whose ADR-001 `layering.toml` registration is an
authored edit), PHASE-09 (evaluation kit). The funnel prescribed `spawn` for
each. PHASE-02's case is not a judgement call but a proof:
`tests/architecture_layering.rs` raises `Unclassified` for a module with no
tier entry **and** `StaleEntry` for a tier entry with no module, so neither
ordering survives — a worker fork building the module is red before it can
commit (the `worker_commit` gate), and a coordinator pre-landing the tier line
is red too. The authored line and the code must land in one commit, and only
the coordinator writes authored state.

## Shape of a fix

A phase-level marker the oracle reads — `dispatchable = false`, or a closed
`execution = "worker" | "coordinator"` field on the plan's phase entry — so
`next` prescribes *execute solo* rather than *spawn*, and the funnel's
prescription stays literally followable.

Worth checking whether the marker is derivable rather than declared: a phase
whose entire edit set falls under `.doctrine/**` selectors is coordinator-only
by construction. Declared is cheaper and honest; derived is harder to get wrong.

## Interim posture (SL-233, settled with the user 2026-07-29)

Conclude coordinator-only phases by hand: execute solo in the coordination
worktree, `dispatch record-boundary` so `sync --prepare-review` still cuts a
`phase/<slice>-NN` ref (`src/ledger.rs` — `boundaries.toml` is that cut's
input), then flip the runtime phase sheet to `completed`. `dispatch next` then
advances correctly, as it did past SL-233 PHASE-01.

Retooling the dispatcher from inside a live dispatch was declined as scope
drift — hence this item.

Related: [[ISS-274]] (`record-boundary` leaves the index and worktree stale,
found while recording SL-233 PHASE-01's row by this route).
