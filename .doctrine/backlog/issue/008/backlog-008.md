# ISS-008: priority::graph::tests::non_backlog_nodes_carry_no_dep_seq_edges reported flaky under parallel just check

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

A concurrent SL-051 session reported `priority::graph::tests::non_backlog_nodes_carry_no_dep_seq_edges`
failing intermittently under a parallel `just check`, alongside (and at the same
time as) `e2e_priority_golden` × 4 failing on `priority.v1` vs `priority.v2`.

## Two competing hypotheses — not yet decided

1. **Shared-target pollution (likely).** The v1/v2 e2e failures were *confirmed*
   to be the documented shared jail-target false-failure — a half-relinked
   `doctrine` binary in `~/.cargo/doctrine-target-jail` served stale/garbage
   output during the SL-050 PHASE-04 worker's concurrent compile. The graph-test
   flake was observed in the *same* window and may be the same artifact (a
   mid-relink test binary), not a real defect.
2. **Ambient-root test isolation (SL-051 agent's diagnosis).** The test was
   characterised as an "ambient-root isolation bug" — i.e. a code path under
   `build(root)` that re-derives a root via a CWD-ascending `root::find`
   (cf. the `no-root-find-walk` gotcha) instead of honouring the explicit `root`,
   so under parallelism it reads the real project corpus.

## Evidence against #2 / for #1

The test (src/priority/graph.rs:655) seeds into an isolated `tmp()` and passes the
explicit `root` to `build(root)`; it looks correctly isolated. It passed clean in
two `just check` runs **after** clearing the polluted `doctrine` fingerprint
(`rm -rf …/.fingerprint/doctrine-*`) and a clean rebuild — 1044 passed, 0 failed.

## Next step

Reproduce deliberately: run `cargo test priority::graph` repeatedly under load on a
clean target. If it survives, close as a shared-target artifact (and the real fix
is hardening the gate against shared-target false-failures, not this test). If it
recurs on a clean target, audit `build`/`scan_entities`/`dep_seq_for` for any
ambient `root::find` and pin the test's root.

Out of SL-050's scope (none of its seven findings); surfaced during SL-050
PHASE-04 (which rewrote this test's dangling assertions onto the consequence
tally). Captured here so it is not lost at SL-050 close.

## Further sighting (SL-124, 2026-06-20)

A `just gate` run during SL-124 PHASE-02 saw two *different* table-driven tests
fail together — `relation::tests::every_variant_appears_in_the_table` and
`relation_graph::tests::overlay_set_equals_resolvable_graph_labels_table_driven` —
then **pass clean in isolation and on the very next `just gate`** (2055 passed, 0
failed). ~60 dispatch worktrees were active at the time (concurrent worker
compiles into the shared jail target). New tests, same flake-then-pass signature,
strongly consistent with **hypothesis #1 (shared-target pollution)** rather than a
defect localised to any one test — the failing set varies by run, which an
ambient-root isolation bug (#2) in a specific test would not explain. Reinforces
that the real fix is hardening the gate against shared-target false-failures.
