# IMP-266: Break integrity coupling via leaf validation contract

Tier-2 (high-value lever) mitigation surfaced by RSK-227. `integrity` is the
symmetric 13/13 nexus of the command-tier tangle — the single highest-leverage
knot in the 23-node core SCC.

## Why integrity is the knot

`integrity` is a validator that **knows every entity kind**. Same-tier edges:

- **out (13)** — imports each kind module it validates: `adr, slice, spec,
  policy, standard, rfc, review, revision, rec, backlog, knowledge, concept_map,
  requirement`.
- **in (13)** — those kinds (and the engine hubs) call back into it.
- **true 2-cycles (4)** — `backlog, rec, review, revision` depend on `integrity`
  *and* are depended on by it.

The bidirectionality is what feeds the core SCC.

## Fix

Invert the dependency (SL-016 pattern, larger scale):

1. Define a per-kind **validation contract** (trait / interface) in a **leaf**
   module.
2. Each kind module implements the contract.
3. `integrity` depends only on the leaf contract, not on each kind module.

Removes all 13 `integrity → kind` up-reaches plus the 4 back-cycles from the
SCC. Expected to collapse much of the 23-node core — the tangle count should
drop well below the current baseline, and `integrity`/`catalog`/`relation_graph`
move closer to being genuine *engine* (see Tier-3 endgame below).

## Payoff / effort

Highest tangle reduction available. Real refactor — **needs its own slice**
(design the contract seam, migrate 13 kinds, keep suites green). Behaviour-
preservation gate applies. **Precedent:** SL-016; ADR-001 gate-edge model
(§"extract the interface to break the cycle").

## Downstream

Enables ADR-001's stated **Tier-3 endgame** — once the engine hubs no longer
reach up into command modules, promote the engine tier to its own workspace
crate so the compiler enforces the boundary (the "when engine churn settles"
trigger). Not this item; this item unblocks it.

Surfaced by RSK-227 (see its `graph/` coupling map). Complements RSK-227's
capture-and-watch arm.
