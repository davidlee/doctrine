# ISS-475: No lifecycle stage settles a slice's own DEC records

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

A slice mints `DEC` records during design; they seed at `proposed`
(`/knowledge`: *"capture seeds the kind's default state"*). Nothing in the
lifecycle then moves them.

- `/design`'s lock attests sections and disposes the review; it does not touch
  record status.
- `/reconcile` and `/close` mention no knowledge-record transition. `/close`'s
  only nearby act is a `REC` with `move = accept` against a flagged
  **requirement**, which is a different axis.
- `/knowledge` owns the verb (`doctrine knowledge status <ID> <STATE>`) but is
  capture-time routing, not a stage obligation.

So the transition is unowned, and the corpus shows both states side by side:
`DEC-103` and `DEC-138` read `accepted`, while the thirteen records `SL-260`
minted (`DEC-263`–`DEC-273`, `DEC-275`, `DEC-276`) still read `proposed` against
a design that has locked. A reader cannot tell an unsettled decision from a
settled one that nobody transitioned.

`SL-260` discharges its own set at `PHASE-03` `EX-7`, which fixes the instance
and not the gap.

What to settle here: whether the transition belongs at the design lock (the
decisions are settled the moment the design they shape locks) or at slice close
(they are provisional until the change lands), and then hang it at that moment
rather than leaving it to an agent's initiative. Adjacent: `DEC-178` (*One
settle verb, reach derived from facet names*), itself `proposed`.
