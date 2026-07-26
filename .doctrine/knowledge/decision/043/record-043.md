# DEC-043: Give observations a dedicated product capability and ledger container

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Observations are raw, cheap, UUID-addressed occurrence signals. They are not
durable synthesized knowledge, actionable backlog work, or numbered evidence
records. Their evergreen governance therefore lives in a dedicated product
capability, PRD-018, and a dedicated observation-ledger container, SPEC-028.

SPEC-028 is a container directly under the Doctrine context, SPEC-003. It is not
a component of the entity engine, because its immutable UUID record model is not
an authored numbered-entity lifecycle. It is not generalized into a reusable
evidence-ledger abstraction yet: one concrete consumer is insufficient evidence
for that abstraction, and “evidence” is already a term of art for EVD knowledge
records.

The design may reuse lower-level infrastructure and service boundaries without
collapsing these semantic distinctions. If a second ledger consumer later
demonstrates the same contract, the shared primitive can be extracted then.
