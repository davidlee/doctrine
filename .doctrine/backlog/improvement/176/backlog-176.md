# IMP-176: Standalone plan.toml validation: malformed plan surfaces late at execution time, not at authoring

Captured from [[mem_019ed002b43d7630b2d69797489e51aa]].

There is no standalone `plan.toml` validation. A malformed `plan.toml` only
surfaces when `slice phases` parses it at execution time — the author gets no
feedback at plan-authoring time. Needs a validation verb or lint-at-save hook;
exact mechanism unverified.
