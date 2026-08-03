# DEC-133: Admission journals endure; forensic exhibits may expire

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Context

Execution capsules can produce large forensic exhibits: Git bundles or
equivalent object archives, worker history, verification logs, metadata, and
permitted transcript material. These exhibits are usually low-value after a
short operational horizon, but become unusually valuable when an incident or
surprising result requires reconstruction.

They are distinct from the trusted-side admission journal. The journal records
what the control plane accepted and why; an exhibit preserves richer material
for later inspection. Making every exhibit permanent authored content would
grow repositories without bound. Making the journal depend on disposable
exhibits would make accepted history dishonest after cleanup.

## Decision

The trusted-side admission journal is durable. It retains the identities,
hashes, verdicts, and archive references needed to state what was admitted,
including the lifecycle state of any referenced exhibit.

Forensic exhibits may expire. They live behind a separately-owned archive
boundary rather than in ordinary authored project content by default. Their
absence after an applicable retention policy has run does not erase or rewrite
the admission journal; the journal represents the exhibit honestly as present,
expired, missing, or otherwise disposed.

The initial design does **not** prescribe a retention duration, quota, or
configuration hierarchy. Project-, slice-, and machine-level policy are neither
required nor precluded. They may be introduced when operational evidence shows
which scope owns a real need. A short retention horizon is the expected ordinary
posture, not a hard-coded product invariant.

Doctrine is not positioning this archive as a cybersecurity product. The goal is
proportionate operational forensics: keep the small audit truth durable, avoid
paying permanent storage cost for routine exhibits, and preserve enough recent
reality to inspect the uncommon event where reconstruction becomes highly
valuable.

Any shipped archive mechanism must still rest on a Doctrine-owned contract under
POL-002; it cannot make correctness depend on a client repository's undeclared
cleanup habits or transient local state. ADR-019's independent asset-policy axes
govern the later choice of storage, publication, and projection mechanism.

## Consequences

- Admission remains explainable after an exhibit expires, but deep reconstruction
  is only promised while the exhibit remains available under the applicable
  policy.
- Large binary evidence does not enter Git merely because it is useful during a
  short forensic window.
- Later retention, quota, and scope configuration can be added without reversing
  the journal/exhibit separation.
- Expiry is an explicit lifecycle event, not silent disappearance.

## Origin

Answers QUE-205, the capsule forensic-archive storage and retention question,
for RFC-025's post-spike cleanup in CHR-053.
