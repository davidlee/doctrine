---
name: spec-product
description: Use when authoring or revising the product specification — the what and why of a capability, upstream of any slice. Use before scoping slices that should descend from a shared product intent.
---

# Spec Product

You are authoring the product spec — the *what* and *why*, upstream of
implementation.

> **Structural (SL-015).** Doctrine manages specs as first-class entities.
> Scaffold with `doctrine spec new product "<title>"`; add requirements with
> `doctrine spec req add <PRD-ref> "<title>" --kind functional|quality`; read the
> reassembled whole with `doctrine spec show <PRD-ref>`; check FK integrity with
> `doctrine spec validate`; list with `doctrine spec list`. Identity + flat fields
> live in `spec-NNN.toml`, the narrative in `spec-NNN.md`, and requirements are
> **peer entities** (`REQ-NNN`) membered via `members.toml`.

Capture:

- the problem and the user/system need it serves
- the desired outcomes and success criteria (the *what*), not the design
- the rationale and constraints (the *why*)
- what is explicitly out of scope

Keep it durable and implementation-agnostic — the *how* belongs in `/spec-tech`
and the per-change design belongs in `/design`. When a product spec is settled,
slices descend from it: scope the change with `/slice`, then `/design`.

If the intent is really a single change rather than an evergreen capability, it
is a slice, not a spec — use `/slice`.
