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
phases, audit) hangs off one stable identity, and linkage, verification, and coverage
can attach without reshaping what already exists.

Once the design is settled, the team also needs an executable account of its current
intent: which ordered phases advance the change, what each phase is responsible for,
what must be true before and after it, and who or what verifies completion. That plan
must govern execution without pretending to predict every discovery. Its history must
remain auditable while obsolete meaning stops governing, and both humans and agents
must be able to identify the current contract without reconstructing it from Git or
conversation history.

## 2. Scope

In scope:

- Declaring a change as a contract — its context, scope, objectives, non-goals, and a
  précis of how "done" is recognised — distinct from the design that realises it.
- A durable, human- and tool-resolvable identity for each change, stable across its
  whole life and across the addition of sibling artefacts.
- A lifecycle vocabulary that tracks a change from proposed intent to reconciled and
  closed.
- A durable, ordered phase plan for an accepted slice: phase objectives, entry and
  exit conditions, verification obligations, and canonical governance links.
- Stable phase and criterion identity, criterion-mode semantics, plan validation,
  explicit evolution of changed criteria, active-versus-historical interpretation,
  rendering, compatibility, and migration.
- Canonical spec and requirement linkage and the seams through which verification and
  coverage read the plan without becoming a second source of plan truth.

Out of scope:

- The technical design, architecture, decisions, and validation design of a change —
  those belong to the design artefact, not the slice.
- Runtime execution progress (phase state) — disposable runtime state, not the
  durable contract.
- The storage schema, algorithms, CLI grammar, and component architecture of the
  phase-plan surface — those belong to the descending technical specification.
- Execution-result storage, requirement coverage reconciliation, and selector
  conformance — adjacent capabilities that consume plan facts without being owned by
  the plan content model.
- A semantic change-claim record, worker-discovery protocol, or general repository
  graph; none is implied by governing phase-plan content.
- Mutation surfaces beyond creating and surveying slices (edit, remove, re-slug).

Boundary: the slice owns the *what* and *whether*; the design owns the *how*. A fact
lives in exactly one artefact, and the slice is not a place to restate design. The
plan owns the current ordered prediction of execution and its phase-local criteria;
runtime progress and observed evidence remain adjacent inputs and cannot author the
plan by derivation.

## 3. Principles

- **Declarative, not imperative.** A slice declares the desired end state and its
  boundary; it does not script the steps. The how is execution, recorded elsewhere.
- **One fact, one artefact.** The slice body and its design sibling have a hard,
  non-overlapping edge. Duplication breeds drift, and drift is the disease doctrine
  exists to kill.
- **Authoritative current knowledge, not complete foresight.** An approved plan
  governs execution until an authorized revision changes it. Discovery can challenge
  the plan but does not silently rewrite it.
- **Immutable history, correctible active meaning.** Stable identities preserve what
  was authored; explicit evolution makes obsolete criteria historical rather than
  leaving them falsely governing.
- **Proof mode names the confirmer.** Entry and exit conditions are distinct from
  verification, and verification distinguishes automated test, agent judgement, and
  human acceptance.
- **Unchanged criteria carry zero evolution ceremony.** Lineage exists only when
  criterion meaning changes; preserving an existing criterion requires no shadow
  record or restatement.
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
- The phase-plan content model must remain project- and language-agnostic; projects
  own language-specific executable procedures through the canonical verification
  seam.
- A plan must reference canonical requirements, specifications, verification
  procedures, and coverage facts rather than restating or replacing their authority.
- Existing valid plans must remain readable when the content model evolves; unchanged
  criteria must not require lineage records or manual re-authoring.

Invariants:

- A slice's identity is permanent: it never changes when padding width grows or when
  the slug is edited.
- The slug is never authoritative — tooling resolves a slice only by its id.
- The declarative contract (the what/whether) and the design (the how) never share a
  home; neither restates the other.
- Phase and criterion identifiers are permanent within their authored scope: they are
  never reused or renumbered, including after replacement or relocation.
- The ordered governing criterion set is deterministically derivable; a superseded or
  withdrawn criterion can never be mistaken for current truth.
- Authored plan truth, runtime phase progress, and observed execution evidence remain
  separate; no observed result can silently mutate the plan.
- Canonical linkage remains available even when empty, so verification and coverage
  have a stable attachment point without duplicating their records into plan prose.

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
- A planner can express an ordered phase contract with unambiguous objectives, entry
  and exit conditions, verification modes, and canonical links before execution.
- Invalid or ambiguous plan content is refused before it can govern execution, with
  enough specificity for the responsible actor to correct it.
- A reader can distinguish the governing criterion set from preserved history without
  consulting Git, while replacement, withdrawal, split, merge, and relocation remain
  auditable.
- An existing valid plan retains the same governing meaning under a model upgrade,
  and unchanged criteria incur no migration or lineage work.

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

Planning flow: after design intent is settled, an operator authors ordered phases.
Each phase states its stable identity, objective, entry and exit conditions,
verification obligations, and applicable canonical links. The valid authored plan
becomes the governing execution prediction while progress is tracked separately.

Criterion-selection flow: entry and exit conditions state when a phase may begin and
what state it must leave behind. EN identifies entry and EX identifies exit;
verification uses VT for automated test, VA for agent check, and VH for human
acceptance. A missing or unsupported classification cannot silently acquire governing
force.

Evolution flow: when criterion meaning changes, an authorized actor preserves the
predecessor and records the successor disposition. Replacement, withdrawal, split,
merge, and cross-phase relocation produce a deterministic governing set. An unchanged
criterion remains active without any evolution record.

Read and validation flow: before execution, validation reports malformed identity,
mode, reference, ordering, or evolution structure rather than repairing it silently.
The ordinary read presents the ordered governing plan; a history read preserves prior
criteria and their dispositions without presenting them as active.

Compatibility flow: a valid plan authored before a content-model extension remains
readable with the same governing meaning. When a material migration is required, it
is explicit and deterministic; failure leaves the prior authored truth intact and
reports what prevented migration.

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

The phase-plan contract is proven by exercising authoring and reads against REQ-439,
REQ-440, and REQ-446: phase order, objectives, condition classes, verification modes,
and canonical links are stable and resolve identically for human and agent readers,
while runtime progress remains outside authored plan truth. Identity and evolution
checks prove REQ-441 and REQ-442 by preserving predecessor identity, requiring no act
for an unchanged control, and deriving the expected governing set for replacement,
withdrawal, split, merge, and cross-phase relocation.

Validation and rendering prove REQ-443 and REQ-444 with positive and negative cases:
well-formed plans govern in declared order; malformed identities, modes, references,
cycles, cardinalities, and active ordering are refused with actionable diagnostics;
and historical criteria never render as governing. Compatibility fixtures prove
REQ-445 by reading existing valid plans without changed meaning and by showing that
any required migration is deterministic and failure-safe. Ownership review proves
REQ-447 by confirming that canonical intent, executable verification, observed
evidence, coverage, and selector authority are referenced rather than re-authored in
the plan.

Where a check must reference a specific obligation, cite the durable requirement
entity (REQ-NNN), never a mobile membership label. Coverage of the functional and
quality requirements is tracked against those entities, not duplicated here.

## 8. Open Questions

(Resolved — reference form. Prose, commits, and comments cite entities by their
prefixed canonical id (`SL-020`, `PRD-010`, `REQ-060`, …); the id is identity, the slug
is never authoritative, and the durable id is cited, never a mobile `FR-`/`NF-`
membership label. The canonical statement lives in the conventions block of
`CLAUDE.md`/`AGENTS.md`; this OQ resolves by reference rather than restating it.)
- OQ-1 — Distributed identity collision across separate working trees or clones is
  not closed by the single-tree claim. Resolving it needs a shared reservation
  authority; until that exists, cross-tree concurrent creation can still collide at
  merge. What is the acceptable interim posture for multi-team work?
- OQ-2 — Should criterion evolution have its own phase-local act identity, or can
  predecessor/successor sets and disposition carry all required audit history? This
  blocks the descending technical schema, not the product behaviour.
- OQ-3 — What ordering rule preserves a deterministic active plan when a criterion is
  relocated across phases or participates in a split or merge? The technical spec must
  settle the rule without weakening REQ-441 or REQ-442.
- OQ-4 — Which content-model upgrades require an authored migration and which can be
  served by a compatibility read? The technical spec must define the boundary and the
  failure-safe transition required by REQ-445.
