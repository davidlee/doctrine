# QUE-174: Evergreen home for friction observations

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-231 introduces friction observations as a dedicated occurrence-evidence
primitive rather than storing every raw occurrence as memory or backlog work.
The existing canon governs adjacent parts separately:

- PRD-004 / SPEC-007 own durable scoped memory.
- PRD-009 / SPEC-015 own actionable work intake.
- PRD-010 / SPEC-019 own citable epistemic and governance records.
- SPEC-024 provides the closest immutable, attributed, session-of-one ledger
  precedent, but descends from estimation/value product intent.

Before SL-231 design locks, determine whether friction-observation collection:

1. extends an existing product and technical specification;
2. requires a new product/technical specification pair; or
3. is a reusable evidence-ledger capability that should be governed above the
   individual consumers.

The answer must preserve the semantic boundary between raw occurrence evidence,
consolidated reusable knowledge, and actionable work, while avoiding a parallel
identity/storage implementation.

## Answer

Friction-observation collection requires a new product/technical specification
pair: PRD-018 governs observations as a raw-signal capability, and SPEC-028
governs the observation ledger that stores, corrects, resolves, and queries
them. SPEC-028 is a container under the whole-system context, SPEC-003.

The capability is deliberately not placed under memory, backlog, knowledge
records, comparisons, or the entity engine. Nor is a generic reusable
evidence-ledger abstraction authored yet. The observation ledger supplies the
first concrete case; extraction should wait for another conforming consumer.
