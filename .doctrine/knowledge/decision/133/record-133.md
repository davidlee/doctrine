# DEC-133: Admission journals endure; forensic exhibits may expire

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Context

Execution capsules can produce large retained artifacts: Git bundles or
equivalent object archives, worker history, verification logs, metadata, and
permitted transcript material. After work is integrated and closed these are
forensic exhibits, usually low-value after a short operational horizon but
unusually valuable when an incident or surprising result requires
reconstruction.

`DEC-137` identifies an earlier lifecycle state that this decision originally
failed to distinguish. A harvested capsule whose result is unresolved is still
**live work**, not merely an exhibit. Letting its retention clock expire would
make recoverability depend on timing rather than correctness.

They are distinct from the trusted-side admission journal. The journal records
what the control plane accepted and why; an exhibit preserves richer material
for later inspection. Making every exhibit permanent authored content would
grow repositories without bound. Making the journal depend on disposable
exhibits would make accepted history dishonest after cleanup.

## Decision

The trusted-side admission journal is durable. It retains the identities,
hashes, verdicts, and archive references needed to state what was admitted,
including the lifecycle state of any referenced exhibit.

Before integration and formal closure, the original harvested capsule is live
work under `DEC-137`. V0 freezes and retains it; it does not automatically
expire, evict, or garbage-collect it. A superseding attempt does not by itself
make the source disposable. Cleanup requires successful integration and closure
of the result or an incorporating repair, or an explicit operator abandonment.

After that lifecycle boundary, the capsule and its bundle become forensic
exhibits and may expire. They live behind a separately-owned archive boundary
rather than in ordinary authored project content by default. Their absence after
an applicable retention policy has run does not erase or rewrite the admission
journal; the journal represents the exhibit honestly as present, expired,
missing, or otherwise disposed.

The initial design does **not** prescribe a retention duration, quota, or
configuration hierarchy. Project-, slice-, and machine-level policy are neither
required nor precluded. They may be introduced when operational evidence shows
which scope owns a real need. A short retention horizon is the expected ordinary
posture, not a hard-coded product invariant.

Doctrine is not positioning this archive as a cybersecurity product. The goal is
proportionate operational forensics: keep the small audit truth durable, avoid
paying permanent storage cost for routine exhibits, and preserve enough recent
reality to inspect the uncommon event where reconstruction becomes highly
valuable. The distinction does not weaken the capsule trust boundary: a frozen
capsule receives no additional authority, trusted code does not execute its
contents, and formal repair runs in a fresh capsule.

Any shipped archive mechanism must still rest on a Doctrine-owned contract under
POL-002; it cannot make correctness depend on a client repository's undeclared
cleanup habits or transient local state. ADR-019's independent asset-policy axes
govern the later choice of storage, publication, and projection mechanism.

## Consequences

- Pending work remains reconstructable until integration and closure or explicit
  abandonment. After that point, admission remains explainable after an exhibit
  expires, but deep forensic reconstruction is promised only while the exhibit
  remains available under the applicable policy.
- Large binary evidence does not enter Git merely because it is useful during a
  short forensic window.
- Later retention, quota, and scope configuration can be added without reversing
  the live-work/journal/exhibit separation.
- Expiry is an explicit lifecycle event, not silent disappearance.

## Origin

Answers QUE-205, the capsule forensic-archive storage and retention question,
for RFC-025's post-spike cleanup in CHR-053. Refined by DEC-137 to place the
live-work boundary before the forensic-exhibit lifecycle begins.
