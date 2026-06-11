# IMP-027: Deferred doc/* canon has no tracked home or enforcement after tech-spec backfill bifurcates the doc

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

The SL-021 tech-spec backfill is retrospective by charter: it captures only
*shipped how*, and under the F4 interim-authority rule (design §9) the shipped
sections of a `doc/*` note are demoted to seed once a tech spec owns them. That
**bifurcates** the source docs. `doc/entity-model.md` is the worked example: its
shipped half (storage rule, entity-vs-facet, identity/reference, Rust model) is
now owned by SPEC-004; its deferred half (Thesis, Migration stance, Adjudication —
self-labelled "Status: direction, deferred. No action now") stays live in
`doc/*` with **no tracked home and no enforcement seam**.

Deferred prose is not an adherence surface — exactly the "untrusted prose"
PRD-012 §1 exists to kill. Genuinely-accepted-but-unbuilt canon needs promotion
to a tracked kind:

- load-bearing global decision → **ADR** (boot-projected, `/inquisition`-checked);
- decided target architecture → **forward-intent tech spec** (SPEC-001/002
  precedent), requirements `pending`/`planned`, distinguishable from verified
  (PRD-013);
- invariant a future agent must honor → **memory** (retrieval gate).

But the *automated* enforcement seams are themselves still forward-intent: the
reconciliation engine (PRD-013 / SPEC-002, draft/unbuilt) that flags
planned-vs-observed, and the drift ledger (IMP-022 / `doc/drift-spec.md`,
unbuilt) that flags normative-spec-vs-observed-code. Today adherence rests on
promotion + cultural gates (`/canon`, `/inquisition`, design-descends-from), not
tooling.

## Scope of this item

Not the backfill (SL-021, retrospective-only) and not retiring `doc/*`
physically (the §9 F4 / §10 follow-up flagged for SL-021 `/close`). This item is
the narrower, durable question: **after backfill bifurcates a doc, where does its
deferred-but-canonical half live, and what enforces it?** Likely needs a triage
rule (per deferred section: promote to ADR / forward-spec / memory, or mark
explicitly non-binding) plus a decision on whether enforcement waits on the
reconciliation + drift capabilities.

## Links

- Surfaced during SL-021 PHASE-02 (exemplar trio authoring), user challenge
  2026-06-11.
- Adjacent to the SL-021 `/close` follow-up (design §9 F4 / §10: `doc/*`
  retirement, out of scope) — see at close-out.
- Enforcement depends on PRD-013 / SPEC-002 (reconciliation) and IMP-022 (drift
  ledger), both unbuilt.
- Related: IMP-008 (reconcile skill + audit/reconcile seam).
