# DEC-073: Section review uses content-bound runtime attestations

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Section review is represented by lightweight attestations in the gitignored
design-run snapshot, separate from the section records established by DEC-072.
An attestation identifies the section, its content fingerprint, the reviewer
lane (`human` or `adversarial`), reviewer attribution, verdict, and any compact
finding dispositions. Changing the section body changes its fingerprint and
stales every attestation against the previous content.

Each run has a small review policy declaring the required reviewer lanes and,
when both lanes are required, their intended order. This directly supports
human-only review, adversarial-only review acting as a human proxy, and both
human and adversarial review in either order. It is policy over orthogonal
review evidence, not another workflow state or a general approval expression
language.

Iterative section findings remain runtime state by default. A finding is
promoted to a durable knowledge record or authored review (RV) ledger entry
when its consequence outlives the iteration, crosses section boundaries, or
must be carried into closure. The integrated adversarial review required by
DEC-066 remains a closure-grade authored RV review; section attestations do not
silently satisfy or replace it.
