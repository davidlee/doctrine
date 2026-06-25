# Plan — SL-155

## Rationale

Two phases, separated by risk profile and test scope.

**PHASE-01 — Cluster A one-liners** is the low-risk prelude: seven edits across
six files, each a single-line comment or const change. No new functions, no new
tests needed (the existing gate catches regressions). The supersede edge
authoring (G5b) is a one-shot CLI command with no code impact. Done cleanly,
this phase gates green in minutes.

**PHASE-02 — Revision list verb** is the substantive work: ~200 new lines in
`src/revision.rs` including constants, `read_revs`, `key`, `list_rows`,
`run_list`, `ListRow`, `json_rows`, the CLI enum variant + dispatch arm, plus
12 unit tests. This phase has its own TDD cycle (red → green → refactor) and
its own gate check. File-disjoint from PHASE-01 (only `src/revision.rs` is
touched here), so the phases can proceed independently once PHASE-01 lands.

The split honours the principle of small, composable, single-responsibility
units. If a template comment fix breaks, the revision list verb is not
re-executed. Each phase gates independently.

## Sequencing

1. PHASE-01: Apply one-liners, author supersede edge, gate.
2. PHASE-02: TDD the revision list (red tests → implementation → green → refactor → gate).

## Test strategy

All new tests are `#[cfg(test)]` unit tests in `src/revision.rs`, following the
existing revision test pattern. No CLI golden tests — the list verb's integration
surface is small enough that unit tests covering all paths (empty tree, filtering,
hide-set, JSON, columns, unknown status/column, round-trip) suffice. CLI golden
pattern is reserved for higher-risk verb surfaces (e.g. `show` mutations).

## Criteria alignment with design

The design is the verification authority. The plan's criteria map 1:1 to the
design's verification table:

| Plan criterion | Design source |
|---|---|
| PHASE-01 EX-1 | Design Cluster A code-impact table (7 edits) |
| PHASE-01 EX-2 | Design EX-05 |
| PHASE-01 EX-3 | Design EN-01, EN-02 |
| PHASE-02 EX-1, EX-2 | Design EN-03 (constants + 12 tests) |
| PHASE-02 EX-3 | Design CLI integration (List variant + dispatch) |
| PHASE-02 EX-4 | Design EX-01, EX-02 |
| PHASE-02 EX-5 | Design D2, EX-03 (tags opt-in via --columns) |
| PHASE-02 EX-6 | Design EN-02 |
| PHASE-02 EX-7 | Design EX-06 (JSON includes tags) |
| PHASE-02 EX-8 | Design EX-08 |
