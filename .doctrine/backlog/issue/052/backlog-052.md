# ISS-052: pi/codex dispatch arm: conformance-registry write never fires; SHAs stranded in the dispatch ledger

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

RFC-004 v0.1 records each phase's source-delta into the **arm-neutral
conformance registry** at `.doctrine/state/slice/NNN/boundaries.toml` (primary
tree). The two dispatch arms feed it differently (RV-157 F-1):

- **claude** — `dispatch record-boundary` double-writes the committed dispatch
  ledger **and** the conformance registry in one call.
- **codex/pi** — has no `record-boundary`; relies on a *separate*,
  orchestrator-issued `slice record-delta` per landed phase
  (dispatch-subprocess SKILL.md:43-46).

SL-153 (pi-driven, first dispatch slice since SL-147) shows the codex/pi path
broken in practice: `.doctrine/dispatch/153/boundaries.toml` (the dispatch
ledger) holds PHASE-03/04 SHAs, but **no `state/slice/153/boundaries.toml`
exists in any worktree or the primary tree**. The `slice record-delta`
conformance write never fired. `slice conformance 153` → `unavailable — no
recorded source deltas`.

## Impact

Conformance is silently unavailable at audit for codex/pi-dispatched slices.
The SHAs exist (in the dispatch ledger) — they're just in the wrong file. The
separate orchestrator-issued `record-delta` step is too easy to skip; it is a
documented instruction, not an enforced funnel beat.

## Direction

Make the codex/pi `integrate` beat mirror its dispatch ledger into the
conformance registry automatically (close the arm-asymmetry of RV-157 F-1),
rather than leaning on a separate orchestrator call. Alternatively, let
`slice conformance` fall back to the dispatch ledger when the conformance
registry is absent — but the unified write is the cleaner fix. Also seed
design-target selectors as part of the dispatch authoring flow: SL-153 had
**no selectors declared**, so even with boundaries present the diff has no
declared baseline.
