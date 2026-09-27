# IMP-495: Repo-wide secret scanning for authored corpus

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Observation `019fc0bc`: live API keys were spliced into a committed review
ledger through a shell-interpolated prose argument. SL-268 removed that route
for the review kind (D10: `-` stdin and `@path` prose arguments, no shell pass),
and deliberately did **not** add secret scanning there — a per-kind scanner
guards one write path while every other authored entity, memory and
observation stays open.

Wanted: one repo-wide check that authored doctrine state (`.doctrine/**`,
shipped `memory/`) carries no credentials — e.g. a `check`/pre-commit leg over
the corpus, fail-closed, naming the file and span. Design picks the detector
and where it runs.
