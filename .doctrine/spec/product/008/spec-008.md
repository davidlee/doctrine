# PRD-008: ADRs

## 1. Intent

A team governing a codebase makes architecture decisions that outlive any single
change — which way the modules layer, what the storage tiers mean, whether a new
class of artefact is sanctioned. The *result* of such a decision lands in code and
docs, but the **reasoning** — the forces that made it necessary, the alternatives
weighed, the consequences knowingly accepted — evaporates into chat logs and commit
messages. Months later an agent finds the rule but not the why, cannot tell a
deliberate constraint from an accident, and either cargo-cults it or quietly
violates it. There is nothing durable, project-global, and citable to reconcile a
later change against the standing decisions of the project.

An **architecture decision record** answers this need: it is doctrine's durable,
project-global record of one significant decision — the context that forced it, the
choice made, and the consequences accepted — captured before the reasoning is lost
and preserved unaltered as the project evolves. Its value is that rationale becomes
a first-class, reviewable, addressable artefact: a future agent inherits *why* a
constraint exists, a decision can be cited from anywhere by a stable identity, its
standing (still in force, superseded, deprecated) is legible at a glance, and the
relationships between decisions have a home that grows without rewriting history.

## 2. Scope

In scope:

- Recording one project-global architecture decision as a durable artefact: its
  context, the decision, the consequences (positive, negative, neutral), how it is
  verified, and its references.
- A permanent, project-global identity for each decision, stable across its whole
  life and across the addition of relationship links.
- A status lifecycle vocabulary that tracks a decision from proposed through a
  terminal standing (accepted, rejected, superseded, deprecated) without altering
  the recorded reasoning.
- Surveying the project's decisions and their standing, narrowable by status.
- A reserved seam for cross-decision relationships — supersession, related
  cross-references, and free-form tags — present from the start, inert until wired.

Out of scope:

- The technical design or implementation of whatever the decision governs — an ADR
  records the *decision and its rationale*, not the mechanism that realises it.
- Enforcing or executing a decision — an ADR is a record, not a gate; nothing in it
  blocks code or runs a check.
- Activating the relationship seam: supersession links and tag-based classification
  are reserved shape, not behaviour, until a later capability wires them.
- Per-change scope, design, plans, or runtime progress — those are slice concerns,
  not project-global decisions.

Boundary: an ADR owns the *decision and its why* at project-global scope; it is not
a per-change contract (that is a slice) and not a design (that is the how). The
record is durable rationale, never an enforcement mechanism.

## 3. Principles

- **Rationale is the payload.** An ADR exists to preserve *why*; a record that
  states the decision but not the forces and consequences behind it has failed its
  purpose.
- **Records are append-only in spirit.** A decision's history is not rewritten when
  it changes standing — its reasoning is preserved and its status moves; supersession
  links to a new record rather than editing the old one.
- **Identity is the integer, not the name.** A decision's identity is its numeric
  id; the slug is a convenience alias and carries no authority or ordering.
- **Status is authored, not inferred.** A decision's standing is a deliberate,
  recorded act, not derived from activity elsewhere; a no-op transition changes
  nothing on disk.
- **The structure anticipates relationships without building them.** Supersession,
  related links, and tags occupy a reserved seam from the first record, attached
  later without reshaping the artefact.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded as
requirement entities and appear under the synthesized Requirements section below.
This section carries only the constraints and invariants that bound every valid
implementation.

Constraints:

- Identity allocation must be collision-free for concurrent agents sharing one
  working tree, without relying on a lock, daemon, or central authority.
- The status vocabulary is a closed set; new standings are added deliberately, never
  improvised per record.
- A status transition must preserve the record's authored prose and its inert
  relationship seam untouched — only the standing and its timestamp may move.
- A record must remain usable under both git and jj project roots; no mechanism may
  be specific to one VCS.

Invariants:

- A decision's identity is permanent: it never changes when padding width grows or
  when the slug is edited.
- The slug is never authoritative — tooling resolves a decision only by its id.
- The recorded rationale is durable: changing a decision's standing never alters its
  context, decision, or consequences.
- The relationship seam is always present, even when empty, so future supersession
  and tagging machinery has a stable attachment point.
- A no-op status transition leaves the record byte-for-byte unchanged.

## 5. Success Measures

- An agent or reviewer can, from a decision record alone, state what was decided,
  why it was forced, and what consequences were accepted — without recourse to chat
  history or commit archaeology.
- A standing decision is citable from prose, commits, and sibling artefacts by a
  stable identity that survives slug edits and padding growth.
- The standing of every decision (in force, proposed, superseded, deprecated,
  rejected) is legible from a single survey, narrowable to one status.
- Two agents creating decisions concurrently in one working tree never collide on an
  identity, and never need a lock or daemon to avoid it.
- A decision's reasoning survives a change of standing intact: superseding or
  deprecating it never rewrites why it was made.
- Relationship links (supersession, related, tags) attach to an existing record with
  no change to its on-disk shape or identity.

## 6. Behaviour

Primary flow — record a decision: an operator names a decision; the system reserves
the next free project-global identity, materialises the record's durable home seeded
for authoring its context, decision, consequences, verification, and references, and
reports where it lives. The record opens in the proposed standing.

Primary flow — survey decisions: an operator asks for the project's decisions and
receives them ordered by identity, each showing its standing, alias, and title; the
survey can be narrowed to a single status.

Lifecycle flow: a decision advances from proposed to a terminal standing — accepted,
rejected, superseded, or deprecated. The transition is an authored act that records
the new standing and stamps when it moved, leaving the rationale prose and the
relationship seam intact.

Guard — idempotent transition: setting a decision to the standing it already holds
changes nothing; the record is left untouched rather than re-stamped.

Guard — malformed record: a record missing its scaffold-seeded standing fields is
refused rather than silently repaired, so a transition never corrupts a hand-damaged
file.

Edge cases and boundaries: an empty project yields the first identity; growth past
the default padding width widens new identities without disturbing existing ones;
editing a slug by hand can leave its alias stale while the canonical identity remains
authoritative; the reserved relationship seam round-trips untouched on every read and
write while it remains inert.

## 7. Verification

Verification confirms that a decision record durably carries its rationale, that
identity is stable and collision-free, and that the status lifecycle and survey
behaviours hold — without binding the spec to a particular implementation.

Identity behaviour is proven by exercising allocation directly: an empty project
yields the first id, a second creation lands the next id monotonically, padding
renders consistently and grows past the default width without renaming existing
records, and a contended claim drives a recompute-and-retry that lands the next free
identity. Durability of the rationale is proven by confirming a created record
persists its structured standing and its authored prose across reads, and that a
status transition moves only the standing and its timestamp while the context,
decision, consequences, and the inert relationship seam survive intact. The survey
behaviour is proven by confirming records render ordered by identity with their
standing, and that narrowing to a status filters correctly, including the empty
result when no record holds it. The idempotence guard is proven by confirming a
no-op transition leaves the record unchanged, and the malformed-record guard by
confirming a record missing its seeded standing fields is refused, not repaired.

Where a check must reference a specific obligation, cite the durable requirement
entity (REQ-NNN), never a mobile membership label. Coverage of the functional and
quality requirements is tracked against those entities, not duplicated here.

## 8. Open Questions

- Should prose and commit messages adopt a prefixed reference shorthand for
  decisions (mirroring the slice convention) for greppability, or remain bare
  numeric? This blocks any convention that cites a decision from outside its
  directory.
- The relationship seam (supersedes / superseded_by / related / tags) is authored
  but inert. When supersession is wired, is the reverse `superseded_by` link derived
  from the forward `supersedes`, or independently authored? This blocks the design of
  the supersession verb and whether the two links can ever disagree.
- Decisions are project-global by definition, yet some rationale is genuinely
  framework-global (travelling with the binary to every client, like an orientation
  memory). Is there a class of framework-level decision distinct from a
  project-local one, and where would it live? This blocks reuse of the ADR shape for
  framework governance.
