# IMP-191 review RV-211: slice status read-only query form — test guardrail and --note UX

When adding a read-only branch to an existing write verb via Option<T>: (1) always add a variant-coverage guard test for any parallel enum-duplicating array, (2) reject write-only flags (--note) when the STATE argument is absent rather than silently ignoring them.
