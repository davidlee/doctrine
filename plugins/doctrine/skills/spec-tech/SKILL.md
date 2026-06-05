---
name: spec-tech
description: Use when authoring or revising the technical specification — the how of a capability, downstream of the product spec and upstream of per-slice design. Use to record durable architecture and mechanism, not a single change's design.
---

# Spec Tech

You are authoring the technical spec — the *how*, downstream of the product spec
(`/spec-product`) and upstream of per-slice design (`/design`).

> **Structural (SL-015).** Doctrine manages specs as first-class entities.
> Scaffold with `doctrine spec new tech "<title>"`; add requirements with
> `doctrine spec req add <SPEC-ref> "<title>" --kind functional|quality`; read the
> reassembled whole with `doctrine spec show <SPEC-ref>`; check FK integrity with
> `doctrine spec validate`; list with `doctrine spec list`. Identity + flat fields
> (incl. `c4_level`, `sources`) live in `spec-NNN.toml`, the narrative in
> `spec-NNN.md`; requirements are **peer entities** (`REQ-NNN`) membered via
> `members.toml`; tech-only spec→spec edges are hand-authored in
> `interactions.toml`.

Capture the durable architecture and mechanism:

- the shape of the solution — components, boundaries, data flow, invariants
- the key technical decisions and their rationale (link relevant ADRs)
- interfaces and contracts that outlive any single change
- constraints the implementation must honour

Keep it evergreen. A single change's concrete design — current vs target
behaviour, code-impact, verification — belongs in that slice's `/design`, not
here. If a decision is project-global and load-bearing, it may be an ADR
(`doctrine adr new`) rather than spec prose.
