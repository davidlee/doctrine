# IDE-036: Surface unknown_supersedes load warnings

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

SL-213's resolution layer collects `Resolution.unknown_supersedes`
(`UnknownSupersedesTarget { row, target }` — a judgement whose `supersedes`
names a uid absent from the corpus, e.g. after a hand-merge drops a session
file). Design R2 mandates these are load-time warnings, not errors, and the
data is doc-commented on `Pipeline` — but no surface consumes it: neither
`compare list`, `findings`, nor `explain` discloses a dangling supersedes
target. Candidate future disclosure: a findings line or a list annotation.
Origin: SL-213 PHASE-06 worker hand-back; RV-266 synthesis.
