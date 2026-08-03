# DEC-124: Contracts ride the refusal and a stage-entry receipt

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The channels

**The refusal** carries the remedy. `GateNotCleared` names the missing conditions
and nothing else — `IMP-390`'s original complaint — and the error channel has no
byte budget, so pairing each unmet condition with what it requires and what act
discharges it competes with nothing.

**A stage-entry receipt** carries the contract itself, edge-grained: on entering a
stage, the contracts for that stage's outbound edge, delivered once. This is
proximity to the time of action — an agent learns what the edge will want *before*
spending a turn failing at it — and it is the register `Fragment` already occupies
(`design.rs:1832-1851`).

**The turn envelope gets nothing new.** `ENVELOPE_NORMAL_BUDGET_BYTES = 24576`,
enforced by an eviction ladder terminating in `EnvelopeIrreducible`, and
`clearances` tokens already ride it uncapped per section row. Contract prose here
would compete with everything else the turn carries.

**Pull is deferred, not rejected.** `doctrine verify` has one member today, and
`DEC-101` left *"which shipped checks it launches with beyond research-current"*
open at phase-plan. A per-condition check — *would I pass, and if not why* — is a
strong candidate and costs nothing to an agent that does not ask. It is a new
family member with its own membership test and census assertion, so it is named as
the phase-plan decision `DEC-101` already anticipated rather than committed to
here. See `DEC-123`.

## No digest

`Fragment`'s receipt re-sends on digest change. A contract has no such change to
detect: it is a pure function of the binary — `DEC-122`'s prose asset plus
`DEC-123`'s const row, both compile-time, composed by `DEC-123`'s injection rule
without touching run state. All ten were walked; none is parameterised by the run.

The only mutation path is editing guidance and recompiling mid-conversation, and
at that point snapshot compatibility is the governing question, not receipt
freshness. Specifying a digest ahead of a real change case is machinery for a
scenario that does not exist. If one emerges, `Fragment`'s mechanism is there to
borrow.

**The dynamic/static boundary is clean, which is why this holds.** What varies per
run is the *complaint* ("you claimed `governing-context-recorded` against
`sec-01`") and the *state* ("`inq-4` and `inq-7` are outstanding"). Both already
belong to the refusal. The contract does not move.

## Addressing grain and delivery grain are different questions

Only one of them is forced, and it is forced by the modelling rather than chosen.

**Per-condition addressing is required by `DEC-067`.** `cumulative_conditions`
walks `Stage::ALL.windows(2)` and accumulates every edge below the target
(`gate.rs:212-226`), so at `reviewing → locked` the cumulative set is **all ten**.
An agent can therefore fail at the last edge on `governing-context-recorded` —
a condition from the first. If contracts were addressable only as edge-blobs, that
agent either re-receives three edges it already holds or cannot reach the one
contract it needs.

`DEC-123` reinforces this from the other side: the rendered contract is composed
per condition anyway, each pulling its own const fields. Per-condition is the unit
of composition regardless of how it travels. It is also `DEC-122`'s per-condition
addressability, which that record kept as its one concession to `IDE-047` and
`IMP-375`.

**Delivery is then free to be edge-grained** — two conditions on three edges, four
on `reviewing → locked`. One receipt at stage entry rather than one per condition,
which answers the tool-call-tax concern without compromising addressing, because
it was never the same question.

Related: `DEC-122` (per-condition addressability), `DEC-123` (the const/prose
split and the injection rule; the deferred pull channel), `DEC-120`, `DEC-121`,
`DEC-067` (cumulative revalidation — the constraint that forces the grain),
`DEC-064` (the envelope's named limits), `IMP-390`, `IDE-047`.
