# ASM-011: Inbound edges are a sufficient proxy for the knowledge shaping an entity

## The known imperfection

The assumption is held with a measured miss, not blind.

`SL-244`'s `design.md` cites roughly ten `DEC` records it holds **no edge to** —
`DEC-063` (which `shapes SL-233`), and likewise `DEC-065`/`066`/`067`,
`-073`/`074`, `-086`/`088`, `-101`/`102`. These are inherited governing context:
cited in the prose because they bind, unlinked because nobody authored the edge.

A composed read built on edges therefore shows a reader the twelve records that
point at the slice and not the ten it argues from.

## Why it is still held

Closing the gap on the read path would mean scanning prose for citations, which
`SL-246` rules out as an explicit non-goal: a read that parses prose inherits
every ambiguity of prose, and the defect is an *authoring* omission. The repair
belongs upstream — a validate-and-warn pass that notices a cited-but-unlinked
record at authoring time (`IDE-009` carries that canary).

## What would falsify it

Not the existence of unlinked citations — that is already known and priced. It
fails if the **unlinked** citations turn out to carry the load: if a reader of
the composed output regularly reaches a materially different understanding from
the edge set than from the union of edges and citations. `validation_plan`
names the check.
