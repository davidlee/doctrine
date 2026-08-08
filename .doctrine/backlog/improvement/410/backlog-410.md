# IMP-410: Gate requirement prose completion: prevent empty Statement/Rationale at status transitions

## Problem

The inflection at REQ-310 shows a process change occurred, but the tooling didn't
enforce it — the earlier 309 REQs were scaffolded and left empty. Without a gate,
nothing prevents the same pattern from recurring.

## Proposed fix

A gate (likely in `doctrine check commit` or `doctrine check gate`, or a
`spec req` status-transition validator) that refuses to mark a requirement
`active` (or `accepted`) when:

- The MD Statement section is empty (only HTML comments / whitespace), **and**
- The TOML `description` is also absent

A Rationale-only emptiness check would be a warning, not a hard gate — some
requirements are self-evident enough that a Rationale is nice-to-have.

## Alternatives considered

- A `doctrine check` lint rule for empty REQ prose
- A skill-level prompt in `/spec-product` that nudges the agent to fill prose
  before marking complete
- Auto-closing the requirement creation loop in `spec req add` (prompt the agent
  for Statement/Rationale at creation time rather than deferring)

## Related

- CHR-059: the backfill of the 335 existing empty REQs
- `scripts/find-empty-reqs.sh`: detection tooling
- IMP-096: Requirements capture and refinement skills for the reconcile loop
- IMP-097: Altitude assessment framework for requirement placement
