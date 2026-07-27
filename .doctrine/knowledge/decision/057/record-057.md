# DEC-057: Exact local recovery with durable semantic reconstruction

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-233 guarantees exact continuation across compactions and local sessions while
the design run's `.doctrine/state/**` runtime state survives. If that disposable
state is lost, Doctrine reconstructs accepted semantic context from linked
DEC/QUE/ASM records, but does not claim cursor-perfect recovery of the
provisional inquiry map.

The run must expose a compact, stable reference and a sufficient resume
projection for a fresh session. `/handover` should first ensure the run and
durable semantic checkpoints are current; when that projection is complete, it
should return the run reference instead of synthesising a parallel prose
handover. Free-form handover content is reserved for genuinely residual context
that the domain model cannot yet represent.

More generally, structured domain-appropriate state should drive unstructured
handover content towards zero. A recurring residual is evidence of a missing
field, checkpoint, or projection—not a reason to make prose packets canonical.
Portable exact recovery after runtime deletion remains outside v1.
