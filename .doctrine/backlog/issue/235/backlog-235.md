# ISS-235: link error message duplicates role-bearing labels

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine link SL-226 specs PRD-016` (illegal label) replies:

    Legal labels: references, references, references, supersedes, governed_by, related, fulfils

`references` appears once per role (implements|originates_from|concerns) but
the roles themselves are not shown, so the list is both duplicated and less
informative than the follow-up error (`references requires a role — author it
with --role <implements|originates_from|concerns>`). Dedupe, or print
`references(--role implements|originates_from|concerns)` once.
