# SPEC-029: Design run engine

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

A container sitting between the design behaviour's thin adapter above and the
entity engine, reservation, knowledge records, and prompt cascade below. It is
the *how* for PRD-019: the mechanism by which a design workflow acquires
durable coordination state, recoverable identity, and a bounded read model.

The container splits along the project's pure/imperative line. A pure run model
holds the stage vocabulary, the forward-boundary gate table, the inquiry-map
types, regression semantics, and the serialization contract; it computes over
values and injected derived facts and reads no clock, randomness, repository, or
filesystem. A thin shell around it performs persistence, fingerprinting,
reservation, materialisation, and rendering. The split is what makes the
workflow's semantics exhaustively testable without a fixture repository, and it
is the reason the gate table is a `const fn` table modelled on the existing
review ledger's permission table rather than a second bespoke matcher.

The container owns one active run per slice. Its snapshot lives in the runtime
state tier: gitignored, disposable, and reconstructible only weakly from
authored records — exact resume is the snapshot's job, and the authored records
support a deliberately weaker reconstruction.

## Responsibilities

Mirrors the structured `responsibilities` list. In summary, the container owns:
the pure run model; the schema-versioned snapshot with revision compare-and-swap,
submission idempotency, and bounded receipt eviction; the authored-design
watermark and its three observation points, the third of which bounds what the
guard warrants rather than extending it; the reserve-then-journal creation protocol;
the reserved-materialisation seam in the entity engine's fresh-creation path;
the design prompt pack; the canonical turn envelope and its bounded projections;
and the command family carrying the sparse mutation contract.

Two ownership boundaries are worth stating because they were contested:

- **The prompt pack belongs here, not to the cascade spec.** The cascade owns
  composition, precedence, and seal; this container owns one closed,
  design-specific fragment store that the cascade delivers. The cascade spec
  gains exactly one sealed slot for the invariant stage fragment and nothing
  else.
- **The knowledge-record surface is reused, not extended.** Accepted design
  outcomes become records of the existing kinds. The record surface acknowledges
  a managed design run as a legitimate provenance for records created through
  the reserve-then-journal protocol; it does not acquire any of this container's
  mechanism.

## Concerns

**Recovery is the dominant failure concern.** Three distinct losses have to be
survivable and they have different answers. Loss of context is answered by the
resume projection. Loss of the runtime snapshot is answered by weaker semantic
reconstruction from authored records — enough to continue, not enough to resume
exactly. A crash mid-creation is answered by ordering: the identity is journalled
before authored bytes exist, so recovery resumes the reserved id rather than
minting a second one and leaving an orphan.

**The abandoned write is a precise guarantee, and the imprecise version is
wrong.** When the authored design moves between validation and the snapshot
write, the write is abandoned. The guarantee is that *the run does not advance* —
not that nothing was written. Effects journalled earlier in the ordering
deliberately remain and stay recoverable; a formulation that promises nothing was
written invites a cleanup path that deletes authored knowledge, which is the one
thing the creation protocol forbids.

**Concurrency is single-writer but not therefore trivial.** One run per slice
removes multi-writer contention, but stale-context writers remain: an agent
resuming from an old projection, a retried submission, a delegated proposal
returning late. Revision compare-and-swap and submission identity are what make
those safe, and both must refuse rather than merge.

**Token cost is a correctness concern, not a tuning one.** The ordinary
projection is the thing delivered every turn, so its size is the protocol's
running cost. Limits are named constants in one module; the bounding evidence
requires a fixture that exceeds every limit before projection, because a bound
whose fixture never reaches it is unproven.

**Two seams touch shared machinery and must stay additive.** The
reserved-materialisation midpoint sits in the entity engine's fresh-creation
path, and the prompt pack rides the existing cascade. In both cases the existing
suites are the proof of behaviour preservation: they stay green unchanged, and
editing one to accommodate the new seam is a failure rather than a fix.

## Hypotheses

- **H1 — a snapshot is the right recovery primitive.** Exact resume is assumed to
  need a schema-versioned, revision-guarded snapshot, with authored records
  supporting only weaker reconstruction. If authored records turn out to be
  sufficient for exact resume, the snapshot is redundant weight.
- **H2 — the existing epistemic kinds fit.** Decisions, questions, and
  assumptions are assumed to be the natural durable sink, with no new kind
  required. A demonstrated semantic mismatch — not an inconvenience — would
  overturn this.
- **H3 — a design-specific prompt store is the right first move.** The cascade is
  general, and forking a closed design-specific store trades reuse for a bounded
  blast radius while the shape is still unknown. This is a deliberate deviation
  from "reuse and generalise", and it is bounded by test rather than by
  intention: the store must have no consumer outside this container.
- **H4 — one active run per slice is sufficient for v1.** Concurrent runs against
  one slice are assumed unnecessary. If they become necessary, run identity is
  already explicit, but the single-run assumption is baked into entry checks.

## Decisions

- **D1 — the gate table is a `const fn` table, not a matcher.** The review
  ledger's permission table is the precedent; a second bespoke matcher would be a
  parallel implementation of a solved problem.
- **D2 — stage, inquiry lifecycle, cursor, posture, review, delegation, and
  recovery are separate types with derived facts between them.** Folding them
  into one hierarchical state machine is the accidental-complexity failure this
  decision exists to prevent, and it is also how provisional state and accepted
  truth end up sharing a store.
- **D3 — node provenance is a first-class field, not a convention.** Whether a
  branch was user-directed or agent-proposed has to be a type-level property, or
  the distinction erodes the first time a restructuring is convenient.
- **D4 — blocked standing is derived, never stored.** A stored flag drifts from
  the graph that justifies it.
- **D5 — the pre-write window is reachable by a deterministic injectable hook.**
  The re-check before the write is only meaningful if a test can drive an edit
  into that exact window; without a seam the claim is untestable and any test
  asserting it proves nothing. The hook is test-only and does not alter the
  production path.
- **D6 — one canonical envelope, three renderings.** Prompt, machine-readable,
  and human status are projections of one type. Three independently maintained
  models would drift, and the drift would be invisible because each looks
  self-consistent.
- **D7 — the snapshot path derives from the existing runtime-state root.** A
  second path literal is a standing violation that guarantees eventual
  divergence; the helper is the single source.
