# QUE-188: Section-level human and adversarial review policy

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

How should SL-233 represent and require section-by-section review so that a
human, an adversarial agent, or both can review a section, with adversarial
review occurring before or after human review or acting as its proxy?

The evidence must be bound to the reviewed content fingerprint so that edits
invalidate stale approval. The design must decide:

- whether each review is a lightweight runtime attestation distinct from
  section content;
- how a run declares which reviewer lanes are required and, when both are
  used, their preferred order;
- whether section-level findings remain runtime state or enter the authored
  review (RV) ledger; and
- how section review composes with the existing integrated adversarial review
  required before the design can be locked.

Avoid turning v1 into a generic approval-policy language or producing one
closure-grade RV ledger per iterative section review unless that durability is
actually required.
