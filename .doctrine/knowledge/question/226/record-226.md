What may a shipped artifact ground a claim on?

The sweep's per-site decision needs this settled, because "delete the id" and
"delete the claim" are different answers and the corpus needs one rule:

1. **A published address only** (`reference/<name>.md`) — resolvable in every
   client repo by construction. ISS-309's current leaning. Costs: the published
   set must be curated, and it only reaches corpus-internal referents.
2. **Prose that stands without any reference** — ground the fact inline; no
   vocabulary needed. Costs: duplication across docs, and the reason *why* a
   decision was made is often exactly what the id was carrying.
3. **A widened vocabulary** — some durable form a client can resolve (a
   published ADR set? a described capability?). ISS-309 leaves this explicitly
   open ("whether shipped assets should be able to cite *anything* durable").

Tension to resolve: most sites are one-clause rationales where inlining is right
and cheap, but a minority (`install/doctrine.toml.example`'s per-knob ids;
`design-prompts/inquiring.toml`'s two load-bearing runbook steps whose reasoning
lives only in a private sketch) carry reasoning that has no home yet. Those need
a destination, not a deletion — otherwise the sweep trades a collision for a
silent loss of rationale.

Related: ADR-005 (skills route, reference docs explain), ADR-019 (publication vs
projection). Carried by SL-267.
