# ISS-492: Catalog scan callers drop warning diagnostics

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`catalog::scan::scan_entities` collects `CatalogDiagnostic`s, but most callers
drop the warnings. `src/commands/relation.rs:331` suppresses every `Warning`
except the empty-relation count. `src/commands/design.rs:1782`
(`shaping_questions`) passes `&mut Vec::new()` and discards them all, and the
catalog's own tests do the same. A degraded read that surfaces only as a warning
diagnostic therefore never reaches the caller: an STD-003 gap across every
kind, including dangling refs.

Surfaced by RV-396 `F-3` (SL-268). SL-268 pushes an RV vocabulary-defect
warning onto this channel, which `doctor` reads (`src/doctor_checks.rs:85`),
and leaves caller-side surfacing to this item. Shape: audit every
`scan_entities` caller and decide per surface whether warnings are rendered,
counted, or explicitly out of scope, with a test per surface.
