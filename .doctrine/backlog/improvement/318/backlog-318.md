# IMP-318: Persist attested coverage on the verification stamp

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why

SL-230 D10 takes the **weak reading** of `verified_sha`: it asserts that
everything git *could observe* about the claim was committed and unchanged at
that commit — not that every declared scope entry was observed. A memory scoping
a path git has never tracked (`.claude/skills/**`, `.harness/probe/**`) is
attested over a proper subset of what it declares.

That reading is deliberate and measured — refusing instead would make an entire
class of harness memory permanently unattestable (SL-230 D10, RV-307 F-21/F-25).
Its residual is that **the shortfall never reaches the stamp's consumer**. It is
reported on stderr at verify time and raised by `validate` as a corpus-health
finding, but neither is attached to the record, so retrieval ranking and
staleness see an unqualified clean attestation and cannot distinguish a full one
from a partial one.

32 active items currently carry at least one unobservable scope entry.

## What

Record, on the attestation itself, what surface it actually covered — at minimum
a flag distinguishing full from partial coverage, more usefully the declared
entries that did not contribute.

## Why it is not in SL-230

It needs a **new persisted field**, so it is a schema change: SPEC-007's
verification axis, the scaffold default and the read path all move. That is
OQ-3's shape and OQ-3's cost, and SL-230 is a body-write slice.

## Relations

- Carried as SL-230 **R8**.
- Raised by RV-307 **F-25**.
- Same schema-change class as SL-230 **OQ-3** (body digest at verify time); the
  two should be scoped together, since both add a field to the same axis and
  both exist to make an attestation say more than "a sha".
