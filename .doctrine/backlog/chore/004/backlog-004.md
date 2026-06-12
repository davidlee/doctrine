# CHR-004: close skill text is stale (pre-ADR-009 lifecycle), needs reconcile against slice status state machine

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

The `/close` skill text predates ADR-009 (slice lifecycle state machine and
conduct axis). It does not reflect the `doctrine slice status <id> <state>`
transition verb (advance/back-edge/skip/abandon over
`proposed→design→plan→ready→started→audit→reconcile→done`), the closure-seam
ordering refusals (`→reconcile` only from `audit`, `→done` only from
`reconcile`), or the D-C9b close-gate (refuse closure while an RV targeting the
slice carries an unresolved `blocker`).

Reconcile the skill prose against the shipped state machine: replace any
hand-edit-status guidance with the transition verb, and align the close ritual
with the audit→reconcile→done seam.

Source: `plugins/*/skills/close` (skill source-of-truth is `plugins/`, not the
gitignored `.doctrine/skills` installed copy — see
mem.pattern.distribution.skills-source-vs-installed).
