# REQ-258: Author cross-kind dependency and sequence edges feeding the derived view

## Statement

A work-like entity — a slice, or any backlog kind in PRD-009's set — can author its
own hard `needs` (prerequisite, payload-free) and soft `after` (sequence, per-edge
`rank`) edges onto another work-like entity. These authored edges are the cross-kind
capture surface the graph-derived priority/actionability view (PRD-011) consumes: a
`needs` edge surfaces its source as blocked until the target settles; an `after` edge
contributes to derived sequence ordering — identically to the backlog item→item
instance (REQ-097), which this generalises rather than replaces.

The valid endpoints are a **closed allowlist**: slice and the backlog kinds, and no
others. Every other admitted kind — governance (ADR/policy/standard), spec,
requirement, knowledge records, drift records, and review/reconciliation records — is
refused as a target at author time, alongside a free-text (unresolvable) target and a
self-edge, each with a clear message. The authored edge names (`needs`/`after`) stay
decoupled from the engine's `dep`/`seq` overlay species — no graph-engine vocabulary
leaks into the authored entity. The PRD-009 backlog item→item capture instance
(REQ-097) is unchanged by this generalisation.

## Rationale

Doctrine governs work across many kinds, but only the backlog could author dependency
and sequence intent (REQ-097). A slice that cannot land until another slice does had
nowhere to *say so* — the dependency lived only in prose. Generalising the same
`needs`/`after` capture surface to every work-like kind lets operator-stated intent
feed the derived view uniformly, which is the whole premise of PRD-011: one comparable
"what's next, and why?" across kinds.

The endpoint allowlist is the load-bearing constraint, and it is stated as an
allowlist on purpose. A denylist enumerated by example rots — the corpus admits more
kinds than any prose list remembers (drift, review, reconciliation records all sit in
the graph), and a "refused: governance, spec, …" list silently permits whatever it
forgot. The allowlist is total by construction: slice + backlog kinds in, everything
else out.

**Why governance/spec/requirement are not work-like targets.** Letting a slice author
`needs → REQ-NNN` or `needs → ADR-NNN` would quietly turn the work-dependency surface
into a *governance-gates-work* mechanism, conflating "I depend on this decision being
settled" with "this is a peer work item I'm sequenced behind." That linkage already
has a home: a slice **descends from** a requirement (`descends_from`), it does not
*need* it as a work prerequisite. Keeping `needs`/`after` to work↔work preserves the
distinction — descent/membership edges carry governance lineage, `needs`/`after` carry
work sequencing.

**This refusal is scoped, not absolute.** A non-work entity legitimately gating
downstream work — an open question gating a slice's design, a held assumption gating an
idea, an active constraint gating a requirement (PRD-010 §3, "a record shows what it
affects") — is a real and intended expression. It is simply a **different surface**: a
non-actionable *gating* edge with its own status-class, where the gating entity blocks
its dependents without ever becoming pickup-able work. That surface is deliberately not
this one (it must not put a knowledge record into the actionable worklist), and this
requirement's refusal anticipates it rather than precluding it. It is tracked as
IMP-047 (trinary actionability), sequenced after the cross-kind capture machinery this
requirement defines.
