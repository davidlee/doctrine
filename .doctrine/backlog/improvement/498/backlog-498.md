# IMP-498: Route presence beyond the design-run lock

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

SL-270 (DEC-326) made the design-run lock refuse while a disposed `blocker` or
`major` finding on the current review pass has no known route. The check reads
that pass once, at lock. Three cases escape it:

- severe findings left open past the lock (`reviewing.md` permits this);
- findings on superseded passes;
- findings disposed after the lock.

DEC-334 reports these apart during CHR-082's trial window. Nothing owns the
question beyond it (RV-400 F-1, RV-401 F-3; owner decision 2026-09-27).

## Decide

Once CHR-082 reports RFC-026 E15: if P10 survives, decide whether presence
should be enforced where the finding is disposed (for example, `review dispose`
and `amend` require `--route` on severe design-facet findings) rather than only
at the lock. DEC-331 dropped facet-gated refusal on the evidence then
available; revisit it on E15. If P10 is killed, close this item with the axis.
