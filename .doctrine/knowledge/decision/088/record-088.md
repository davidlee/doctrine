# DEC-088: Accepted checkpoints carry content-bound user attestations

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

A managed checkpoint separates semantic content from the authority claim that
permits accepted design truth.

For a new or adopted DEC/ASM to satisfy an accepted checkpoint gate, the sparse
declaration includes a user-acceptance attestation with:

- `authority = "user"`;
- a concise `basis` identifying what the user accepted;
- an optional harness turn reference when available.

Doctrine derives and binds the attestation to the checkpoint payload
fingerprint, inquiry disposition, and current run revision. The agent does not
supply the digest. Without a current attestation, creation uses the knowledge
kind's default status and cannot satisfy an accepted-decision gate.

The submission still comes through the coordinating agent. The attestation
records that the agent asserts explicit user acceptance; Doctrine does not claim
to authenticate a human independently in v1.

`basis` remains required in v1 because it makes the authority claim inspectable,
but its value is empirical rather than assumed. CHR-049 should measure whether
it assists review/recovery or mostly repeats the preceding answer and behaves as
paperwork tax.
