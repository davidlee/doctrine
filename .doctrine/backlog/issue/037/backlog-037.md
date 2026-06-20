# ISS-037: relation/relation_graph table-driven tests flaky under parallel just gate

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

A `just gate` run during SL-124 PHASE-02 (2026-06-20) failed two table-driven
tests together:

- `relation::tests::every_variant_appears_in_the_table`
- `relation_graph::tests::overlay_set_equals_resolvable_graph_labels_table_driven`

Both **passed clean in isolation** (`cargo test --bin doctrine relation`) and on
the **immediately following `just gate`** (2055 passed, 0 failed). ~60 dispatch
worktrees were active at the time — concurrent worker compiles into the shared
jail target (`~/.cargo/doctrine-target-jail`).

## Read

Same flake-then-pass signature as ISS-008 (priority::graph), and the *failing set
varies by run* (priority::graph there, relation/relation_graph here). A variable
failing set is consistent with **shared-target pollution** (a mid-relink test
binary serving stale/garbage output during a concurrent worker compile), NOT with
an ambient-root isolation bug localised to one test. So this is most likely the
same artifact as ISS-008 hypothesis #1, surfacing on different tests.

## Not caused by SL-124

SL-124 changed only `src/boot.rs` (exec-path resolver + hook-merge normalize) —
nothing under `relation`/`relation_graph`. The tests are deterministic on a clean
target.

## Next step

Do not chase a fix in the named tests. The durable fix is **hardening `just gate`
against shared-target false-failures** (e.g. retry-on-flake, or per-run target
isolation for the gate) — track that with ISS-008, which this `after`-links.
Cheap interim: re-run `just gate` once before treating a table-driven test failure
as real.
