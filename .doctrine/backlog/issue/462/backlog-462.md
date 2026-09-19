# ISS-462: check plan cannot express verification by unchanged pre-existing test

`doctrine check plan` requires every verification row's `test_file` to be
declared by a **`design-target`** selector. That rule has no room for a
legitimate and explicitly-governed pattern: verification by a pre-existing test
that the slice must leave *untouched*.

AGENTS.md names the pattern as a rule of the road — the **behaviour-preservation
gate**: "when changing shared machinery (the entity engine), the existing suites
are the proof — they must stay green unchanged."

## Specimen

SL-246 PHASE-01 collapses two per-kind facet projections onto one authored
table and must not move a byte of `knowledge show` output. Its `VT-2` verifies
this against `tests/e2e_knowledge_cli_golden.rs`, and its `EX-7` states that the
suite must be "green UNCHANGED — a diff there is the alarm, not a golden to
update."

The slice therefore declares that path deliberately as **`scope-relevant`**,
noted "C2's instrument — must stay green UNCHANGED; a diff here is the alarm."

`doctrine check plan SL-246` exits 1 on it:

```
PHASE-01  VT-2  UndeclaredTestFile — test_file `tests/e2e_knowledge_cli_golden.rs` declared by no design-target selector
```

and prints the remedy `doctrine slice selector add SL-246
tests/e2e_knowledge_cli_golden.rs --intent design-target`.

## Why the printed remedy is wrong

Following it asserts the slice *intends to change* a file whose whole role is to
fail if changed — inverting `EX-7` and erasing a distinction the slice author
made on purpose. The two intents carry different meanings and the tool's repair
collapses them.

## The shape of a fix

The check conflates "file carrying a verification" with "file this slice intends
to change". A verification-by-unchanged-test wants to satisfy the declaration
requirement from `scope-relevant` — either by admitting that intent for `VT`
rows, or by a distinct intent (or a `VT` row marker) naming the
behaviour-preservation case.

Left unfixed, the cost is an agent being told, by the tool, to make a
semantically false declaration — and an unresolvable red at audit for any slice
that uses the pattern. Note `check plan` is **not** covered by `doctrine check
gate`, so it does not block phase execution.

Surfaced during SL-246 PHASE-02 (capsule-driver).
