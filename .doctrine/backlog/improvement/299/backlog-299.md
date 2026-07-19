# IMP-299: Detect requirement-entity vs spec §-prose drift

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

A spec has two tiers that must agree: the `REQ-NNN` requirement **entities**
(`description` + `acceptance_criteria`) and the spec's **§-prose** (Intent,
Principles, Behaviour, Verification) narrating the same obligations. Nothing
checks that they agree. `spec validate` proves FK integrity, not tier
consistency — so a requirement rewrite silently leaves the §-prose describing the
superseded obligation, and the drift is invisible until an external reviewer
greps for it.

Observed twice on PRD-017 / SPEC-026: codex round 1 fixed the requirement
entities; round 2 was spent entirely re-aligning stale rule-derived-licence and
resolver/manifest language still sitting in the PRD's section prose. A whole
review round to catch what tooling could flag.

## Possible directions

- A lint that flags terms present in a spec's §-prose but absent from (or
  contradicted by) its requirement entities — even a coarse "these nouns appear
  in prose but no live requirement" heuristic would have caught the manifest /
  resolver / predictable-licence leaks.
- A product-tier smell check: mechanism nouns (`manifest`, `resolver`, `embed`,
  `runtime-loaded`, `backing source`) appearing in a `kind = "product"` spec's
  prose or requirements, with an allow-list for ADR-019 storage *vocabulary*
  named in §4 constraints (the licit exception).
- Related affordance gap: no `requirement edit --description/--criteria` verb, so
  every requirement rework is a full-field TOML hand-edit ([[IMP-298]]); a proper
  edit verb could also carry a "now review the narrative that cites this" nudge.

## Links

Pattern captured as `mem.pattern.doctrine.spec-prose-requirement-drift`. Sibling
of IMP-298 (requirement edit-verb gap). Surfaced via RFC-021 Contract B specs
(PRD-017 / SPEC-026).
