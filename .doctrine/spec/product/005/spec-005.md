# PRD-005: Reservation & Leasing

## 1. Intent

Doctrine courts parallel agents and multiple teams working the same governed
codebase at once. The moment two of them create an entity independently, they
race for the same number — both mint `slice-003`, both believe they own it, and
the collision surfaces only later, at merge, as corrupted history. The same
hazard waits behind every numbered entity kind doctrine grows: slices, specs,
requirements, and whatever comes next. Intent here is to make a freshly allocated
identity *trustworthy* — once an agent holds a number, no other agent can hold
the same number — so concurrent work composes instead of colliding.

A second, related need is to separate *claiming a number* from *the entity
existing yet*. A team should be able to reserve an identity now and author its
contents later, without that reservation being a liability if the work is never
finished. The desired end state is a single coordination primitive that hands out
durable, collision-free identities across whatever reach the project needs — one
working tree today, every clone of a shared remote when teams demand it — while
the entities themselves stay ordinary, visible working-tree files. Its value is
that correctness under concurrency stops being a thing each entity kind reinvents
and becomes a property the platform guarantees once.

## 2. Scope

In scope:

- Reserving a durable, collision-free identity for any numbered entity kind, as a
  permanent claim that decouples holding a number from authoring its contents.
- Arbitrating contended allocation so exactly one agent wins a given identity and
  the rest retake the next free one, without a lock, daemon, or central authority.
- The reach of a claim as a backend choice — a single working tree, or every
  clone of a shared remote — selected by configuration, transparent to callers.
- Surveying the held claims: who holds what, and since when, across the active
  coordination store.

Out of scope:

- Transient, expiring claims for edit-exclusion (TTL, heartbeat, release,
  force-acquire, crash recovery, clock-skew handling) and the per-kind
  write-gating they would serve — a deferred extension of the same primitive,
  specified elsewhere, not part of this contract.
- Storing entity content in the coordination layer — claims reference entities by
  name and never hold their bytes; entities remain working-tree files.
- Guarding direct human edits to working-tree files — reservation guards identity
  allocation, not content mutation.
- Cross-VCS parity beyond the project's chosen version control; where a shared
  remote is unavailable, the contract narrows to single-tree reach.

Boundary: this capability governs *claims on names*, never *content*. It answers
"who holds this identity, with what reach" and nothing about what the identified
entity contains.

## 3. Principles

- **A claim is an atomic exclusive hold on a name.** Every guarantee rests on one
  indivisible operation that succeeds only if the name is unheld; there is no
  second arbiter and no negotiated tie-break.
- **One algorithm, swappable reach.** What a caller asks for never changes when
  the reach of a claim changes; the backend's reach and failure modes differ, the
  reservation logic does not.
- **Coordination is invisible to the record.** Claims never enter the entity's
  content, history, or review surface; the durable record shows entities, not the
  bookkeeping that kept their identities distinct.
- **A permanent claim never expires.** A reserved identity is taken forever; an
  abandoned reservation is a harmless gap, not a fault to recover from.
- **The simplest thing that holds, with the apparatus arriving with its caller.**
  Machinery for hazards that no current caller faces is not built ahead of need;
  the contract reserves the shape so it is deliberate, not retrofitted.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded
as requirement entities and appear under the synthesized Requirements section
below. This section carries only the constraints and invariants that bound every
valid implementation.

Constraints:

- Identity allocation must be collision-free for concurrent agents without relying
  on a lock, daemon, or central coordinating authority.
- The primitive must be blind to entity meaning: any kind, any namespaced key, with
  no knowledge of an entity's layout or contents.
- Reach must be selectable by configuration without changing what callers request;
  where the broader-reach backend is unavailable, allocation must still be correct
  at the narrower reach.
- A claim must reference an entity only by name and must never store its content.

Invariants:

- At most one holder ever holds a given identity; no two agents proceed on the
  same number.
- A reserved identity is permanent — once claimed it is never reissued, even if its
  entity is never authored.
- The coordination record never appears in the entity's content, version history,
  or review surface.
- A successful claim is the single linearization point: an identity is held only
  once the claim is accepted, never on optimistic local state alone.

## 5. Success Measures

- Two or more agents working concurrently never proceed on the same entity
  identity, and never need a lock or daemon to avoid it.
- A team can reserve an identity and author its entity later; an unfinished
  reservation leaves a harmless gap, never a fault that must be repaired.
- When a shared-reach backend is configured, identities are unique across every
  clone of the remote; when it is unavailable, allocation falls back to
  single-tree reach with the reduced reach made visible, not silently assumed.
- An operator can survey the held claims and see who holds what and since when.
- Adding a new numbered entity kind reuses the same reservation guarantee with no
  new collision-avoidance mechanism invented for it.

## 6. Behaviour

Primary flow — reserve an identity: a caller asks for the next identity in a
namespace; the system computes the next free candidate, attempts an atomic
exclusive claim on it, and on success returns a held identity. The reserved number
is durable and is not tied to whether the entity's contents exist yet.

Contention guard: when two callers compute the same candidate at once, exactly one
wins the atomic claim; the loser observes the collision, recomputes the next free
candidate, and retries, until it lands a free identity. No caller ever proceeds on
a contested number.

Reach selection: the reach of a claim is chosen from configuration — a single
working tree, or every clone of a shared remote. When reach is set to resolve
automatically, the broader-reach backend is used when its remote is reachable, and
otherwise allocation falls back to single-tree reach with a one-time signal that
cross-team reach is off. This automatic fall-back governs the structurally
single-tree case — no remote configured. A *configured* remote that fails is
treated as a hard error rather than silently downgraded; the operator opts into
reduced-reach local allocation explicitly. A transient failure can thus never
silently mint a local id that collides with another clone's accepted remote
reservation (SL-148 D8, RV-152 F-3).

Survey flow: an operator asks for the held claims under a namespace and receives
each held identity with its holder and the time it was acquired.

Edge cases and boundaries: an empty namespace yields the first identity; gaps left
by abandoned reservations are skipped, never reused; under single-tree reach,
agents in separate clones can still collide — an accepted limitation made visible,
not silently hidden. A claim never touches the identified entity's content, so a
reservation can exist with no entity authored behind it.

## 7. Verification

Verification confirms that an allocated identity is trustworthy under concurrency,
that reservations are durable and permanent, that reach behaves as configured, and
that coordination stays invisible to the record — without binding the spec to a
particular backend.

Collision-freedom is proven by exercising allocation under contention: an empty
namespace yields the first identity; a claim that loses the atomic race drives a
recompute-and-retry that lands the next free identity; gaps and maxima resolve
correctly; and no two holders ever come away with the same number. Durability and
permanence are proven by confirming a reserved identity persists and is never
reissued, even when no entity is authored behind it. Reach behaviour is proven by
confirming that automatic selection resolves to broader reach when the remote is
reachable and falls back to single-tree reach — with the reduced reach surfaced —
otherwise, and that an explicit reach choice overrides selection. The
coordination-only boundary is proven by confirming a claim references an entity by
name without holding its content and never appears in the entity's record. The
survey behaviour is proven by confirming held claims report their holder and
acquisition time.

Where a check must reference a specific obligation, cite the durable requirement
entity (REQ-NNN), never a mobile membership label. Coverage of the functional and
quality requirements is tracked against those entities, not duplicated here.

## 8. Open Questions

- Single-tree reach cannot see other clones, so teams in separate clones can still
  collide before a shared-reach backend is configured. What is the acceptable
  interim posture for multi-team work while only single-tree reach is available?
  This blocks any guarantee of cross-team uniqueness in the default configuration.
- Resolving reach automatically currently costs a coordination round-trip on each
  reservation. Whether that probe should be cached or amortised is unresolved; it
  blocks tightening the latency budget for high-frequency reservation.
- The permanent claim leaves one durable record per reserved identity. At what
  volume of reserved identities, if any, does that record count become a concern
  that warrants a different allocation strategy? This blocks adopting the primitive
  for any future kind that would reserve identities in very large numbers.
