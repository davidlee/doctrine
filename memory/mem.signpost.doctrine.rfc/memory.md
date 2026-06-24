# Doctrine RFC artifacts signpost

RFC (Request for Comments) artifacts are governance-neutral deliberation
documents. They capture design discussion, proposal rationale, and
community feedback before a decision is formalised as an ADR or spec change.

## When to use an RFC

Start an RFC when a consequential change needs deliberation before it
becomes binding. RFCs are lighter than ADRs — they carry discussion
and alternatives, not architectural authority. Once deliberation
converges, an ADR or spec revision is the binding outcome.

## CLI

The CLI is the source of truth: `doctrine rfc --help`, never guess.
Key verbs: `new`, `list`, `show <ID>`, `status`.

## Where they live

RFCs live under `.doctrine/rfc/nnn/`. Each is a `rfc-nnn.toml` +
`rfc-nnn.md` pair with optional attachments in the same directory.

See [[signpost.doctrine.adrs]] for the binding-decision counterpart,
[[signpost.doctrine.revisions]] for the spec/policy change-axis,
and [[signpost.doctrine.file-map]] for the directory layout.
