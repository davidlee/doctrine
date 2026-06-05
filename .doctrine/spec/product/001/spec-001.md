# PRD-001: Slices

## 1. Intent

A team changing a governed codebase needs to say *what* is going to change, *why*,
*what it touches*, *what risks it carries*, and *what "done" looks like* — and to say
it **before** code moves, in a form that outlives the conversation that produced it.
Without that, intent is stranded in chat history or commit messages, scope drifts
silently, and there is nothing durable to reconcile the shipped change against.

A **slice** answers this need: it is doctrine's primary unit of intentional change —
a declarative change bundle that names the desired end state and its boundary up
front, then carries that contract through the work and into closure. Its value is
that intent becomes a first-class, durable, reviewable artefact: scope is explicit
and defensible against accidental expansion, every later artefact (design, plan,
phases, audit) hangs off one stable identity, and the linkage and coverage machinery
that doctrine will grow can attach to it later without reshaping what already exists.

## 2. Scope

In scope:

- Declaring a change as a contract — its context, scope, objectives, non-goals, and a
  précis of how "done" is recognised — distinct from the design that realises it.
- A durable, human- and tool-resolvable identity for each change, stable across its
  whole life and across the addition of sibling artefacts.
- A lifecycle vocabulary that tracks a change from proposed intent to reconciled and
  closed.
- A reserved seam for future linkage (specs, requirements, supersede relations) and
  the coverage gates that will read it.

Out of scope:

- The technical design, architecture, decisions, and validation design of a change —
  those belong to the design artefact, not the slice.
- Runtime execution progress (phase state) — disposable runtime state, not the
  durable contract.
- Enforced coverage gates, spec/requirement linkage enforcement, and the audit/patch
  lifecycle — reserved seams only until the spec corpus exists to enforce against.
- Mutation surfaces beyond creating and surveying slices (edit, remove, re-slug).

Boundary: the slice owns the *what* and *whether*; the design owns the *how*. A fact
lives in exactly one artefact, and the slice is not a place to restate design.

## 3. Principles

- **Declarative, not imperative.** A slice declares the desired end state and its
  boundary; it does not script the steps. The how is execution, recorded elsewhere.
- **One fact, one artefact.** The slice body and its design sibling have a hard,
  non-overlapping edge. Duplication breeds drift, and drift is the disease doctrine
  exists to kill.
- **Identity is the integer, not the name.** A slice's identity is its numeric id;
  the slug is a convenience alias and carries no authority or ordering.
- **The structure anticipates the future without building it.** Linkage, coverage,
  and audit attach to reserved seams later — never by restructuring the artefact.
- **Reserved vocabulary is recorded deliberately, not retrofitted.** Lifecycle stages
  exist from the start so gating can attach to them later, even while unenforced.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded as
requirement entities and appear under the synthesized Requirements section below.
This section carries only the constraints and invariants that bound every valid
implementation.

Constraints:

- A slice must remain usable under both git and jj project roots; no mechanism may be
  specific to one VCS.
- Identity allocation must be collision-free for concurrent agents sharing one
  working tree without relying on a lock, daemon, or central authority.
- The lifecycle vocabulary is a closed, ordered set; new stages are added
  deliberately, never improvised per slice.

Invariants:

- A slice's identity is permanent: it never changes when padding width grows or when
  the slug is edited.
- The slug is never authoritative — tooling resolves a slice only by its id.
- The declarative contract (the what/whether) and the design (the how) never share a
  home; neither restates the other.
- The reserved linkage seam is always present, even when empty, so future coverage
  machinery has a stable attachment point.

## 5. Success Measures

- An agent or reviewer can, from a slice alone, state what is changing, why, what is
  out of scope, and how "done" will be recognised — without recourse to chat history.
- Scope creep is detectable: a change that exceeds the slice's declared scope and
  non-goals is visibly out of contract.
- Two agents working concurrently in one working tree never collide on a slice
  identity, and never need a lock or daemon to avoid it.
- New artefact kinds (design, plan, phases, audit) and, later, spec linkage attach to
  an existing slice with no change to the slice's on-disk shape or identity.
- A reviewer can survey all slices and their lifecycle standing at a glance.

## 6. Behaviour

Primary flow — declare a change: an operator names a change; the system reserves the
next free identity, materialises the slice's durable home and its contract document
seeded for authoring, and reports where it lives. The slice opens in the initial
lifecycle stage.

Primary flow — survey changes: an operator asks for the slices in the project and
receives them ordered by identity, each showing its lifecycle standing, alias, and
title; the survey can be narrowed to a single lifecycle stage.

Lifecycle flow: a slice advances through its ordered stages from proposed intent, to
accepted-and-scoped, to under-way, to reconciling-what-shipped, to closed. Any stage
in the vocabulary is a valid standing.

Concurrency guard: when two agents reserve an identity at once, exactly one wins the
claim; the loser observes the collision and retakes the next free identity. No agent
proceeds on a duplicated identity.

Edge cases and boundaries: an empty project yields the first identity; growth past the
default padding width widens new identities without disturbing existing ones; editing
a slug by hand can leave its alias stale while the canonical identity remains correct
and authoritative.

## 7. Verification

Verification confirms that a slice durably carries its contract, that identity is
stable and collision-free, and that the lifecycle and survey behaviours hold — without
binding the spec to a particular implementation.

Identity behaviour is proven by exercising allocation directly: empty project yields
the first id, gaps and maxima resolve correctly, padding renders consistently and
grows past the default width without renaming existing slices, and a contended claim
drives a recompute-and-retry that lands the next free identity. The contract's
durability is proven by confirming a created slice persists its structured identity
and its prose contract across reads. The survey behaviour is proven by confirming
slices render ordered by identity with their lifecycle standing, and that narrowing to
a stage filters correctly. The separation invariant is proven by confirming the
contract and the design never restate one another.

Where a check must reference a specific obligation, cite the durable requirement
entity (REQ-NNN), never a mobile membership label. Coverage of the functional and
quality requirements is tracked against those entities, not duplicated here.

## 8. Open Questions

- Should prose and commit messages adopt a prefixed reference shorthand (for example a
  letter-prefixed id) for greppability, or remain bare numeric? This blocks any
  convention that wants to link to a slice from outside its directory.
- Distributed identity collision across separate working trees or clones is not closed
  by the single-tree claim. Resolving it needs a shared reservation authority; until
  that exists, cross-tree concurrent creation can still collide at merge. What is the
  acceptable interim posture for multi-team work?
