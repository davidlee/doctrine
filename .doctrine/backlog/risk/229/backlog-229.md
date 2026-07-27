# RSK-229: Managed behaviours lack an explicit privileged authority contract

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Risk

RFC-021 accepts Doctrine-owned behaviour definitions but explicitly notes that
dynamic tool output does not acquire system-message authority merely by
labelling itself authoritative. It calls for a harness contract and empirical
adoption evidence.

The current generated boot snapshot strongly instructs agents to route and use
skills, but does not establish a general meta-contract saying that a
Doctrine-resolved behaviour definition or current workflow envelope is active
instruction, how it relates to harness/system/user authority, or when it must
be refreshed.

SL-233 may therefore observe weak adoption even if its state model and prompt
decomposition are sound. Without separating the cause, the experiment could
misattribute an authority/delivery failure to the managed-design protocol.

## SL-233 boundary

This is not a v1 blocker and does not widen SL-233 into RFC-021 activation or
harness redesign. SL-233 should:

- retain the privileged `/design` skill adapter;
- make its resolved envelope explicitly self-identifying and imperative;
- evaluate adopt, adhere, refresh, recover, and complete separately; and
- record failures where the agent saw the guidance but did not treat it as
  governing.

## Fast-follow trigger

Prioritise a narrow authority-priming experiment if representative use shows
the adapter activates but agents ignore, subordinate, or fail to refresh the
Doctrine-resolved obligation. Candidate work should test a concise boot or
harness-level authority contract before adding more prompt prose or workflow
enforcement.

A cheap Doctrine-repository experiment is available before a product change:
`.doctrine/governance.md` is user-owned and dynamically incorporated into the
privileged boot snapshot. With user approval, a concise rule can tell agents
that a Doctrine-resolved behaviour envelope is the active repository process
instruction, subordinate to system/developer/user authority and current until
Doctrine refreshes it. This is a dogfood mitigation and experimental arm, not
the general harness contract RFC-021 calls for; it must not be shipped as if it
resolved client-project authority.

This record originates from SL-233 and is governed directionally by RFC-021;
it should be resolved from observed adherence evidence rather than assumption.
