# EVD-001: Attestation survives a committed body edit

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The observation

A memory's verification axis is **not** invalidated when its claim changes.
Observed directly during SL-230's design round, on a live memory, not inferred
from reading code.

`mem.system.memory.global-master-authoring` was verified — stamping
`verified_sha = 933b747c`, `verification_state = "verified"`,
`reviewed = 2026-07-25`. Its prose body was then substantially rewritten and the
rewrite **committed**. The verification axis was untouched: it still reads
`verified`, against a commit that attested different content.

## Mechanism

The axis is three fields — `[review].verification_state`, `[review].reviewed`,
`[git].verified_sha` — written **only** by `stamp_verification`
(`src/memory.rs:3350-3362`). `apply_edit` manages title, summary, status,
lifespan, review_by, trust, severity, key and scopes, and no verification field.
Nothing else writes them. So any edit — through the verb or by hand — leaves a
prior attestation standing.

`memory validate` does carry a staleness check (`src/memory.rs:3424`), but it
counts commits touching the memory's **scoped paths** — the code the memory makes
claims *about* — since `verified_sha`. It never looks at the memory's own item
directory, so a change to the claim itself is invisible to it.

## Why it matters

SPEC-007 § Concerns names this hazard directly ("over-trust of stale or poisoned
memory"), and the retrieval sort ranks verification **above** lexical score. A
stamp that outlives its claim therefore does not merely mislead — it reaches
agent context weighted more heavily than an honest unverified record would be.

Before SL-230 this required hand-editing `memory.md`, so it was a footgun rather
than a trap. Adding a body-write verb makes it a one-command operation, which is
why SL-230 closes it rather than merely noting it.

## Detection is available from existing data

The same plumbing the scoped-paths check uses already answers the question, just
pointed at the memory's own directory:

```
git rev-list --count <verified_sha>..HEAD -- <memory item dir>
```

Returned 3 for the memory above. No new persisted field is required for the
committed case. See [[mem.pattern.memory.verification-axis-is-not-self-invalidating]]
if that memory is later recorded, and QUE-173 for the uncommitted/masters gap
that a digest would close.
