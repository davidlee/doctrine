# IMP-411: Make shipped install assets conform to their governing spec

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Source

Surfaced authoring SPEC-031 (*Phase plan surface*), whose `[[source]]` anchors
include `install/templates/plan.toml` — a shipped asset.

## Problem

A shipped install asset can be the de facto contract for a subsystem while
nothing binds it to the spec that governs it.

`install/templates/plan.toml` is the authored criterion schema every client
project scaffolds from. SPEC-031 now governs that schema and anchors the
template, but the anchor is inert: `spec validate` checks foreign-key integrity
across the corpus and never opens a `[[source]]` path. A template that drifts
from its governing spec — a field renamed, a mode dropped, a rule restated
differently — stays green.

The tempting repair is to cite the governing ids inside the asset. That is
wrong, and was briefly done and reverted: an install asset ships into projects
with no doctrine corpus of their own, so `SPEC-031 REQ-464` in a scaffolded
`plan.toml` is a reference the reader cannot resolve. Shipped assets must stay
self-contained; the binding belongs in the platform, not in the payload.

So the conformance signal has nowhere to live today, and the gap is general:
the same shape holds for every spec anchoring anything under `install/`.

## Intended outcome

Give shipped assets a way to **light up** when they drift from the spec that
governs them, without embedding project-local ids in the asset.

Directions worth weighing, not a settled design:

1. Make `[[source]]` anchors verifiable at all — a check that every anchored
   path exists, as the floor. Anchors currently rot silently, which is a
   corpus-wide precondition for anything richer.
2. A conformance check over anchored *shipped* assets specifically: the
   spec declares what the asset must contain (keys, modes, id forms) and the
   check reads the asset rather than trusting prose on both sides.
3. Reuse the existing selector-conformance machinery if its predicate fits
   rather than growing a second one.

Whatever the mechanism, it must run in this repo against doctrine's own corpus
and must not require the client project to carry doctrine's spec ids.

## Boundaries

- No project-local entity ids in anything under `install/` (POL-002 —
  the product must not ship references only this repo can resolve).
- Do not grow a second conformance predicate if the existing one fits.
- Assess the general anchor-verification gap before building a
  plan-template-specific check; the plan template is the instance, not the
  defect class.

## Links

RFC-029 carries the cluster this surfaced from. SPEC-031 is the spec whose
anchors are currently inert. POL-002 is the constraint that rules out the
cite-the-id shortcut.
