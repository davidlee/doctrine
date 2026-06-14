# IDE-010: Revision kind: model pending intent to revise governance (specs/ADRs) as a first-class entity

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why

Surfaced during SL-060 design (cross-kind dep/seq capture, IMP-033). When work
"depends on" a governance document (a spec or ADR), the honest relation is **not**
a `needs` prerequisite edge — it is a *pending intent to revise that governance*.
Governance docs are evergreen/terminal, not work items; a slice does not wait on a
spec *existing*, it waits on the spec being *changed*. That change-intent is the
missing entity.

So specs/ADRs were deliberately kept OUT of SL-060's `needs`/`after` source set:
shoehorning revise-intent into the dep/seq axis models the wrong thing.

## What (sketch — not designed)

A **Revision** kind: a first-class, work-lifecycle entity capturing a pending
intent to revise a governance target (spec/ADR/POL/STD). It would:

- carry a target edge to the governance entity it proposes to revise;
- be itself actionable work (it gates the dependents that need the revision, and
  settles when the revision lands) — composing with IMP-047's `Gating` class;
- let a slice/backlog item `needs` the **Revision** (work→work, in-axis) rather
  than `needs` the governance doc (work→evergreen, gradient-inverting).

Distinct from supersession (a completed lifecycle flip) — a Revision is the
*pending* intent, before the change exists.

Related: SL-060 (IMP-033), IMP-047 (trinary actionability / `Gating`), ADR-009
(lifecycle), ADR-010 (relation modelling).
