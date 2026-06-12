# Governance superseded_by is ADR-004 §5 sanctioned canon, not an outbound-only violation

ADR-004 §5 is a bounded **carve-out**: a stored reverse edge MAY be co-written on
the target entity when that target's file is rewritten anyway for an independent
lifecycle transition. `superseded_by` is the canonical case — supersession flips
the predecessor to `superseded` status (its file is rewritten regardless), so
co-writing `superseded_by` is **zero marginal coupling** and the *only honest
place* a reader of the dead record finds its successor. ADR-004 Verification
bullet 1 names it *"the sole sanctioned reverse field"*.

**The trap:** it reads like an ADR-004 outbound-only violation, but it is the
explicit sanctioned exception. SL-046 design D4 **and** backlog IMP-032 *both
independently* misread it as a violation to remove. Do not file removal/migration
work against `superseded_by`; the most an honest follow-up can be is a `validate`
cross-check that the stored value agrees with the derived `in_edges` reciprocal.

**Reader nuance:** a relation reader still must not *project* the stored field —
but per ADR-004 **§3** (inbound is the registry-backed surface's *derived* job),
not because the field is illegal. Project the outbound `supersedes`, derive
"superseded by" from `in_edges`. The three §5 conditions (lifecycle transition on
target + target rewritten anyway + only honest home) gate every future reverse
field; pure navigational backlinks satisfy none and stay derived.
