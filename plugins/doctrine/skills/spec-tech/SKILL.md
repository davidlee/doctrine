---
name: spec-tech
description: Use when authoring or revising the technical specification — the how of a capability, downstream of the product spec and upstream of per-slice design. Use to record durable architecture and mechanism, not a single change's design.
---

# Spec Tech

You are authoring the technical spec — the *how*, downstream of the product spec
(`/spec-product`) and upstream of per-slice design (`/design`).

> **Not yet structural.** Doctrine does not manage specs as first-class entities
> yet. Technical specs are evergreen, authoritative prose under `doc/*`, authored
> and committed by hand. There is no CLI scaffold; create the file under `doc/`
> following the existing conventions there.

Capture the durable architecture and mechanism:

- the shape of the solution — components, boundaries, data flow, invariants
- the key technical decisions and their rationale (link relevant ADRs)
- interfaces and contracts that outlive any single change
- constraints the implementation must honour

Keep it evergreen. A single change's concrete design — current vs target
behaviour, code-impact, verification — belongs in that slice's `/design`, not
here. If a decision is project-global and load-bearing, it may be an ADR
(`doctrine adr new`) rather than spec prose.
