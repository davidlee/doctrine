# Concept Map: Workflow

Models the core doctrine workflow at a conceptual level: how work originates
(user intent, backlog, GitHub), flows through the slice lifecycle FSM
(proposed → done), interacts with evergreen specifications (PRD / Tech Spec),
and resolves through explicit reconciliation — never derivation by precedence.

Covers both execution postures (solo serial, parallel dispatch), the human
gates at `Ready` and `Done`, and the support ecosystem (review, memory,
handover, consult) without detailing CLI commands or dispatch internals.
