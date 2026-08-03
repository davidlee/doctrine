# DEC-137: Stale results use candidate admission while source capsules stay frozen

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Context

`QUE-202` asks how the second of two capsule results produced from one accepted
base can be admitted after the first result has moved the accepted ref. `SL-241`
proved that the second result is refused safely and without transferring objects
into the canonical repository, but that refusal is content-blind: it does not
distinguish a clean composition from a real content conflict.

`DEC-110` already requires the target model to reuse the incumbent candidate
layer's conflict and staleness semantics rather than building a parallel
admission system. The reusable semantics are the object-only three-way merge,
the durable `Conflicted` state, admission by an immutable Git commit identifier,
and an expected-tip compare-and-swap at integration. The incumbent shell is not
reusable unchanged because its provenance gate reads a coordination-branch
journal that the capsule model removes.

The recovery mechanism also bears on capsule lifetime. Treating the rejected
result as an expiring forensic exhibit would make correct work disappear because
of cleanup timing. Building a new content-addressed rescue store for v0 would
avoid that loss but add another storage and garbage-collection subsystem before
the worktree-to-capsule migration can land.

## Decision

### Admission reuses the candidate engine, not its coordination shell

After a result from contracted base `B` receives `advance/stale-base` because
the accepted ref is now at `A`, the trusted control plane may start admission of
that pinned result explicitly. It supplies the candidate engine with the current
accepted commit `A`, the pinned source commit `S`, the contracted base `B`, and
the verify-capsule attestation. This replaces the incumbent `Verified` row read;
it does not replace the candidate semantics.

The candidate engine runs its existing object-only three-way classification of
`S` against `A`:

- a clean composition creates a normalized candidate with status `Created`;
- a content conflict records `Conflicted` and halts without admission or
  canonical-ref mutation; and
- neither outcome silently rebases, force-updates, or auto-resolves anything.

The exact normalized or hand-resolved candidate is verified in a fresh
verification capsule. Admission then pins the verified commit identifier. The
trusted journal records the intended transition before one expected-tip
compare-and-swap advances the accepted ref from `A`. If the accepted ref moves
again, a fresh candidate records explicit supersession against the new base and
the loop repeats. Movement of the candidate ref itself is different: a
fix-on-top descendant of its recorded merge may be admitted and must not be
discarded merely because the live candidate ref drifted.

The recorded candidate state is authoritative for callers. `Conflicted` is a
successful classification but a non-continuable admission outcome; no scripted
caller may interpret the incumbent command's zero exit status as permission to
continue. `ISS-305` owns the exact command-line signalling correction.

### A harvested source capsule is frozen live work

For v0, the original capsule is the rescue payload. After harvest it is frozen:
formal execution never resumes in that mutable environment, its authority is
not widened, and trusted code does not execute capsule-authored content. A repair
is a new transaction in a freshly provisioned repair capsule based on the
current accepted commit, with the frozen result supplied as an input. Its output
is a new candidate transaction and is verified separately.

The original capsule remains live work until the result, or a repair that
incorporates it, is integrated and formally closed. A supersession marker alone
does not make it disposable. An operator may explicitly abandon and clean up a
result, but v0 performs no automated destruction of unresolved work.

After integration and closure, the capsule crosses into `DEC-133`'s forensic
exhibit lifecycle and becomes manually cleanup-eligible. Configurable automatic
retention or eviction may be added later, but it must preserve this explicit
lifecycle boundary.

### Keep v0 capacity handling advisory

V0 does not add storage reservation, throughput backpressure, or a separate
rescue archive. Provisioning emits a conspicuous free-space warning using a
simple configurable expectation; an initial heuristic may warn below twice the
expected capsule size. Capacity exhaustion requires manual intervention and
never triggers automated capsule deletion.

The control-plane normalization policy may later choose flattened or full
canonical history. That choice does not affect recoverability before closure:
the frozen capsule retains the original history until explicit cleanup.

## Consequences

- Same-base concurrency has one admission state machine rather than a capsule
  imitation of the incumbent candidate layer.
- The long-term isolation posture remains fresh-transaction based; retaining a
  capsule does not turn it into a persistent worker environment.
- Correct harvested work cannot be lost through automated cleanup timing, while
  v0 avoids a new archive, quota, or garbage-collection subsystem.
- Disk growth is visible and operator-managed in v0. This is deliberately less
  automatic than unbounded predictable cleanup or eviction behaviour.
- Machine or disk loss remains an out-of-band durability concern addressable by
  ordinary backup or replication of capsule storage.
- Repair-chain cleanup needs a mechanically proven incorporation relation or an
  explicit operator disposition; semantic reimplementation cannot be inferred
  equivalent merely from a similar final tree.

## Alternatives considered

- **Resume the original capsule for repair:** rejected because it carries
  mutable environment state across transactions and weakens the fresh-capsule
  isolation model.
- **Expire every harvested bundle as forensic evidence:** rejected because an
  unresolved correct result could disappear due only to retention timing.
- **Build a durable content-addressed rescue store in v0:** deferred as useful
  later policy and infrastructure, but unnecessary while the frozen capsule is
  retained.
- **Import every stale result into canonical Git immediately:** rejected because
  it expands the trusted storage surface and defeats the ordinary stale-refusal
  property that transfers nothing into the canonical repository.
- **Automated quota eviction or pre-reservation:** deferred. Both are additive;
  neither should delay the simpler non-destructive cutover.

## Origin

Answers `QUE-202` for RFC-025 and supplies the conflict, repair, and lifecycle
input required by `REV-046`.
