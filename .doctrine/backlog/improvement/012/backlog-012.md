# IMP-012: Architectural triggers: structural triggering-condition on backlog kinds, sense-checked at a planning gate

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Doctrine has no mechanism for *architectural triggers* — deferred work that
should fire when a condition arises, not on a date. Review/walkthrough advice of
the form "lift the shared shape when you next reshape either list surface" or
"extract the FSM when the lifecycle-transition verb lands" is correct precisely
*because* it is conditional: doing it now is premature, doing it never is debt.
Today that advice has nowhere durable to live — it dies in a transcript. A plain
backlog item loses the *when*; a memory records the fact but does not surface at
the moment of relevance.

The gap: governance can express **what** and **why**, not
**when-conditioned-on-a-future-edit**.

## Relation to PRD-011 — NOT a standalone mechanism

This is not new machinery; it decomposes onto existing product intent. PRD-011
(Graph-Derived Backlog Priority) already owns the derived **actionability** view:
"what should I look at next, and why" — an item is surfaced when its blockers
clear. A triggered-but-unfired item is simply *non-actionable* in PRD-011 terms,
and "fire at the right moment" is PRD-011's next-work surface marking it
actionable. The authored trigger *field* is PRD-009's capture seam (minimal
authored metadata, capture stays cheap). So:

- authored trigger condition → **PRD-009** (a new optional capture facet)
- firing / surfacing at the gate → **PRD-011** derived actionability channel
- the matcher → policy layer (reuse `retrieve.rs` scope predicates), which keeps
  PRD-011's graph core path-free per its constraint ("must not depend on file paths")

### The novel bit: a third trigger species

PRD-011's actionability inputs are **entity-relation-driven** (blocked-by, open
question, dependency, drift — entity→entity edges). Prior art in `~/dev/bough`
adds only a **temporal** trigger (`deferred_until: Option<DateTime>`,
`bough/db/src/types.rs`), which PRD-011 puts explicitly out of scope ("no
time-pressure semantics"). An **architectural trigger fires on a code-surface
edit** — "when a phase touches `src/listing.rs`." The condition references a
*code file*, not an entity and not a timestamp. That **edit/path-glob trigger is
in neither PRD-011 nor bough** — it is this item's actual contribution, and it
fits PRD-011 only as a *policy-layer* input (a non-graph actionability mask),
never as a graph edge. → see Open Question for PRD-011 below.

## Proposed

Give backlog kinds (at least `improvement`/`chore`/`risk`) a structural
**trigger** facet in `backlog-NNN.toml` — the condition under which the item
becomes actionable. Two non-exclusive delivery seams:

- **(a) preboot projection** — open triggers surface in the boot snapshot so
  every session sees pending conditioned work. Always-visible; but fights the
  boot-trim direction (see IMP-007) — triggers must be terse / capped.
- **(b) planning gate** — `/plan` or `/phase-plan` queries open triggers and
  sense-checks "does this phase touch a triggered surface?" Targeted, no boot
  cost; but only fires if the skill remembers to check (soft gate unless wired
  as a workflow step).

Likely answer: structural trigger field NOW (cheap), gate-(b) as the first
consumer, projection-(a) deferred behind the trim work.

### Trigger shape — reuse, don't invent

A free-text trigger is unmatchable by tooling. A **path/glob trigger** is
concrete and matchable: "fires when a phase touches `src/slice.rs` or
`src/spec.rs` list rendering." `src/retrieve.rs` already owns scope-admittance
predicates (path / glob / command / tag matching, SL-008) that decide whether a
memory is in scope for a set of touched paths — the **same predicate engine** can
match a trigger against a phase's planned file set. No new matcher: ride that
seam. A trigger is then `{ globs = [...], note = "..." }`, and the planning gate
runs the existing scope match over the phase's touched paths.

### Design tensions to resolve in the slice

- Soft gate (skill remembers) vs hard gate (workflow step / preflight check) —
  enforceability vs friction.
- How a phase declares its "touched paths" before code is written (planned vs
  actual file set) — the trigger can only match a *declared* surface.
- Trigger lifecycle: does firing auto-transition the item, or just surface it?
- Overlap with memory scope-ranking: is an architectural trigger just a memory
  with a write-path scope, or a distinct work-intake facet? (Likely distinct:
  work vs knowledge boundary — see `using-doctrine.md`.)

## Relations

### Back-conversion worklist (do when this ships)

These items already carry their trigger as `## Trigger` PROSE because the
structural facet did not exist when they were filed. On delivering IMP-012,
convert each to the new structural trigger field:

- **IMP-013** — lift slice/spec list+show shape. Path trigger:
  `src/slice.rs`, `src/spec.rs` (list/show fns).
- **IMP-014** — `listing.rs` cross-verb golden harness. Path trigger:
  `src/listing.rs`.

(**IMP-006** — uniform lifecycle-transition verbs — is a related but *dependency*
edge, not a path trigger: the slice FSM extraction fires when that item is
picked up, not when a file is touched. Leave it as prose.)

These three are also the **acceptance fixtures** for IMP-012: if the mechanism
cannot cleanly express IMP-013's two-path trigger and IMP-014's single-path
trigger and surface them at the right planning gate, it is not done.

No structural backlog→backlog edge exists today (ADR-004 relationships are
`slices`/`specs`/`drift` only) — so the worklist links above are prose. But the
backlog→**spec** edge does exist: this item's `specs = ["PRD-011", "PRD-009"]`
records the decomposition structurally, and IMP-013/014 carry `specs =
["PRD-011"]`. (`backlog.rs:370` already notes the reverse view is "deferred,
PRD-011" — that resolver is where backlog→backlog blocking edges and inbound
trigger references eventually surface.)

### Candidate Open Question for PRD-011 (proposed, not yet authored)

> OQ-009 — Should the derived actionability policy admit a **code-surface
> (path-glob) trigger** as a non-entity actionability input — an item held
> non-actionable until a phase's planned/touched file set matches its trigger
> globs — or is edit-conditioned dormancy a separate mechanism outside the graph?
> The graph core must stay path-free (constraint §4), so a path-trigger can only
> enter as a policy-layer mask resolved by the `retrieve.rs` scope predicates,
> never as a graph edge. Blocks whether IMP-012/013/014 are PRD-011 channels or a
> sibling capability.

Adding this OQ to PRD-011 is a `/spec-product` action — left for the operator.
