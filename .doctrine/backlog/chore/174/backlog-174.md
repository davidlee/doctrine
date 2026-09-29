# CHR-174: Test hygiene: table-driven duplicates and behaviour-stating names

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

RV-411 F-18, F-21 (`doctrine show RV-411`). Low-risk cleanup, best done module
by module alongside other work in the same file:
- F-18: collapse near-duplicate runs into table tests: boot pi-extension
  suites (about 30 tests down to about 9), memory `apply_edit_*` (about 25),
  review `*_requires_note`, the layering `check_*` bite tests, and the
  burndown lifecycle states (only 3 of 9 tested).
- F-21: rename `vt1_`/`phase07_`/`iss232_` style tests to state the behaviour
  they pin. Move provenance (slice/RV ids) to commit messages. Drop stale test
  counts and false claims from module headers.
