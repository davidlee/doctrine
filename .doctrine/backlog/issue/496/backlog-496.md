# ISS-496: reseat dangler scan ignores .toml relation edges

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`scan_danglers` (`src/integrity.rs`) globs `.doctrine/**/*.md` only, so after a
`reseat` it misses citations held in `.toml` relation edges — memories'
`[[source]] ref = "RV-NNN"`, `plan.toml` criteria, `[[relation]]` rows. ISS-279's
RV-323 rehome found six memories and a plan.toml EN-1 invisible to it. Reported
danglers under-count, so a reseat looks cleaner than it is. Split out of SL-269
(`inq-5`); scan `.toml` too, or reuse the relation-edge reader.
