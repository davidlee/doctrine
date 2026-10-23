# DEC-403: Codex MCP registers at user level on plain install

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Amended (consent and codex CLI, 2026-10-23)

User rulings: "your recommendations are solid. I think -y grants consent, but probably the docs for it need to say so. prompt's good. decline: accept your rec. too". Evidence: EVD-033.

Reopens the "no gate" half of the original choice: a write outside the repo is a real decision, so it gets its own question (consistent with DEC-381's rule that only real choices are asked). Declining must not retire the project wrapper, or codex is left with no doctrine server at all. Trust wording rests on the user's observation (EVD-033 item 5): codex's prompt is the generic per-folder trust, not an MCP-specific approval.