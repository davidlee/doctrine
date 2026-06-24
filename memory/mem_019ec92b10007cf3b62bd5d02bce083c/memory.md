# Reading doctrine entities

**Read entities via `doctrine <kind> show`, never judge from one tier.**

This is a worked failure lesson. Doctrine entities (slices, ADRs, specs,
requirements, memories) have two tiers per the storage rule:

- **TOML** — structured, queried data (status, relations, criteria).
- **MD** — prose (rationale, narrative, scope description).

An entity's `.md` body may appear "empty" or "hollow" because the data lives in
its `.toml`. An agent that reads only the `.md` (e.g. opening `slice-NNN.md`)
misses the structured fields and makes incorrect judgments. An agent judged a
requirement "hollow" by reading its `.md` alone — but the `.md` is empty *by
design*; all authority is in the TOML.

**The rule:** before forming a conclusion about any entity, read it through the
CLI via `doctrine <kind> show <ID>` (or `doctrine inspect <ID>` for cross-kind
relation views).

The CLI is the source of truth for command shapes — `doctrine --help`.

See [[concept.doctrine.storage-model]] for the storage rule,
[[fact.doctrine.cli-source-of-truth]] for why guessed flags are stale flags,
and [[signpost.doctrine.reference-docs]] for `using-doctrine.md`.
