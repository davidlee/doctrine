# DEC-406: Codex MCP ownership: user slot by name, project slot is the override

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Amended (codex review, 2026-10-23)

User assent: "yes, agreed" to the SL-286 codex-review dispositions.

F3 (consequence made explicit, decision unchanged): the first install can replace a working, hand-written user-level `command`/`args`. The report says "replaced" and shows old → new. Per-project or deliberate custom commands belong in the project layer.

F5: project-level classification is by form, not by whether the entry works, and the report says so. A `command` with no path separator is PATH-dependent: shadow warning, install incomplete. A `command` containing a path separator (absolute or relative) is an override: reported, not held incomplete. An empty `command`, or `command`/`args` of the wrong type, is malformed: snippet, install incomplete. `CODEX_MCP_WRAPPER_FORMS` has held one form since it was introduced (5288e108a, SL-271), so no earlier wrapper wording exists to retire.

Probe coverage: precedence and `CODEX_HOME` behaviour verified on codex CLI 0.159.1 and daemon 0.163.0/0.164.0 (SL-286 research Thread 3, EVD-032); later versions unverified.

## Amended (consent and codex CLI, 2026-10-23)

User rulings: "your recommendations are solid. I think -y grants consent, but probably the docs for it need to say so. prompt's good. decline: accept your rec. too". Evidence: EVD-033.

Field-level preservation (the original choice) is impossible through `codex mcp add`; it becomes disclosure plus consent. Accepted for: codex owning its own file format and symlink handling, a one-line pasteable fallback, and a structured read.