# IDE-048: Shared guarded-transition model for doctrine's state machines

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The question

Doctrine has more than one guarded state machine. Should they share an internal
model of *transitions and their guards*, rather than each spelling it out?

Raised during `SL-244`'s design, from the observation that naming states while
leaving transitions implicit is backwards: transitions are where guards, triggers
and reasons live, so they are what a state-machine model should make first-class.
`SL-244` acts on the modelling discipline (a closed forward-transition type in
the design run's gate) and deliberately stops short of generalising it.

## Why a third-party FSM crate is probably not the answer

Recorded so a future reader does not re-derive it.

- **Typestate crates are ruled out by persistence.** They make each state a
  distinct type and transitions consume `self`, so illegal transitions do not
  compile. `Stage` is snapshot state — it round-trips through TOML and is
  deserialised into a value nobody knows at compile time. `Run<Drafting>` cannot
  exist when the stage arrives from disk.
- **Runtime-table crates model the trivial part.** They generate a state enum, an
  event enum, and a transition function returning roughly `bool`. The design
  run's forward machine is a five-state straight line with four edges and no
  branching (`gate.rs:194` — total on non-terminal stages, one successor each).
  That is not the hard part.
- **The hard part is the guard contract.** `boundary_conditions(edge)` feeds the
  *renderer* with pending conditions for edges nobody is crossing; conditions
  carry derivation rules, reach, activation, coverage, observed facts, a
  required-actor rule resolved from run data, prose assets, and structured
  refusals that name the missing act. A generic guard is a predicate; doctrine's
  guard is a small document. No FSM library models that.
- `POL-002` and the write-less-code ethos both point at reuse *between doctrine's
  own machines*, not at a dependency whose abstraction is thinner than ours.

## Trigger condition for revisiting

`ADR-009` gives the slice lifecycle as a second state machine. Two instances is
roughly where a shared abstraction starts earning its keep; a third would make
the case properly. Revisit when a third appears, or when either machine grows
branching that its bespoke spelling handles badly.

Related: `ADR-009` (slice lifecycle state machine), `SL-244` (where the
transition-first framing was applied to the design run's gate).
