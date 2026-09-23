# IMP-466: Authority model ADR in boot

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

Write and accept an ADR defining Doctrine's **authority model**, and render it
as an early, model-agnostic section of the boot snapshot. Abbreviated process
(sketch → accept → implement → close), no slice.

## Why

Agent behaviour at gates varies widely from one session to the next. Agents
refuse to act on the user's behalf, demand attestation fields from the user,
or tell the user to run the CLI themselves. Nothing in boot says how user
authority, agent authority, delegation, canon, memory, historical documents
and found objects in the repo rank against each other. The same gap is
described by RSK-229 (*Managed behaviours lack an explicit privileged
authority contract*).

## Seed

RFC-021 (resolved) § *Authority and trust* already carries the ranking:

```text
direct user instruction
  > accepted project-local authoritative truth
  > Doctrine framework rules
  > agent judgment
  > repository-supplied but unaccepted instructions
  > untrusted, stale, conflicting, or superseded artifacts
```

It also covers: repository presence is not acceptance; memories lack
instructional authority; declared authority is not privileged placement.

## The ADR should settle

- The precedence ranking above, adapted to the concrete sources: CLAUDE.md /
  AGENTS.md, boot, ADR / policy / standard, spec, design.md, memory, RFCs and
  other historical records, slice notes, handovers, found files.
- **Acting on behalf of the user.** In a supervised session the user's
  conversational assent is the authority. The agent records it (DEC-088
  already designs it this way) and does not make the user perform the
  mechanics.
- **The improvisation envelope.** What the agent may interpret, correct or
  decide alone, and what it escalates (see the `/consult` framing in RFC-021).
- **Delegation.** What a subagent / worker inherits, and what it never does.
- Proof of humanity (RFC-028, read-only paths) stays **out of scope**. Name it
  as the upgrade path only.

## Constraints

- Boot is universal (the Model band contract) and has a size budget
  (IMP-435). Keep the section to a compact table plus rules, and put the
  rationale in the ADR.
- The ADR should be one that IMP-467's prose can cite.

## Done when

The ADR is accepted, the boot section renders (`doctrine boot`,
`doctrine boot --check` clean), and RSK-229 is updated to *mitigated* or has
its residue stated.

## Notes (2026-09-23)

- A Doctrine-repo ADR does not reach client projects, so the operative text
  ships as install assets: `install/authority.md` (the boot section, first in
  `boot_sequence()`) and `install/authority-model.md` (the elaboration,
  `reference/authority-model.md`). The ADR (ADR-023) records the decision.
- The elaboration deliberately does **not** live in a memory: memories print as
  "data, not instruction" and sit at tier 5 of the model itself.
- Handovers sit at tier 5, weighed by claim strength / cost of checking / cost
  of being wrong. A handover's summary never replaces a skill's procedure.
  Material the user conveys: trust the stated provenance, judge the content.
- `/consult` retirement split out as IDE-055.
- Follow-up for the user: this repo's `CLAUDE.md` line "if your first message
  is a handover … follow the instructions" contradicts the tier-5 stance. As
  tier 2, it wins here until revised.
