# DEC-002: Capture-vs-harvest boundary: SL-214 owns during-work capture, SL-215 owns end-of-work harvest

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Knowledge capture splits on *when*, not *what*: SL-214 ships the `/knowledge`
skill and in-flow touchpoints (design forks, consult outcomes, preflight
assumptions) — records are born the moment they arise. SL-215 owns the
end-of-work harvest surface (wrap-up sweeps that mine notes/audits for records
missed in flight) and consumes `/knowledge` as its sink.

## Rationale

- During-work capture preserves context that harvest can't reconstruct
  (the fork actually weighed, the assumption actually carried).
- Harvest catches what flow-state capture drops; the two are complements,
  not alternatives.
- A single slice owning both would couple a shipped skill's design to a
  harvest mechanism still being specced — the boundary keeps SL-214
  closeable.

## Alternatives considered

- Fold harvest into SL-214: rejected — see coupling above (SL-214 design D3).
- Capture only at harvest time: rejected — loses in-context fidelity.
