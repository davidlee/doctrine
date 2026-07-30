# ISS-282: known-fragment arg carries two grammars

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`design resume --known-fragment <V>` now accepts two different shapes of `V`,
consumed by two different functions in `src/commands/design.rs::run_resume`:

| shape | consumer | effect |
|---|---|---|
| bare name, e.g. `inquiry` | `fragment_lines` | reports agreement against `run.fragments` — today always `NOT held by this run` |
| `name@digest` | `fragment_section` | binds a receipt; a current digest omits the fragment body |

A bare name reaches `fragment_section` and parses as `None` (deliberately —
`parse_receipt` refuses a receipt that binds no bytes), so it never suppresses a
body. A `name@digest` value reaches `fragment_lines` and is reported verbatim as
`known_fragment inquiry@<digest> NOT held by this run`, which reads like a
warning about the receipt the caller just correctly supplied.

## Why it shipped this way

SL-233 PHASE-07 F-14: `tests/e2e_design_projection.rs` asserts
`fragment_lines`' `NOT held by this run` line, so the behaviour-preservation gate
required T4b to **extend** alongside it rather than replace it. One arg growing a
second grammar was the cheapest thing that satisfied both. Disclosed at the time
rather than discovered later.

## The compounding factor

`fragment_lines` compares the caller's declarations against `run.fragments`,
which is **always empty** — `FragmentReceipt`/`FragmentGroup` are staged but
never written (PHASE-07 F-12). So its answer is unconditionally "NOT held",
which is why the confusing line is easy to miss: it is constant, not wrong.

Resolving the staged-but-unwritten receipt store would make `fragment_lines`
meaningful and this collision sharper, not softer. Sequence accordingly.

## Options

1. Split the surface: keep `--known-fragment <name>` for the agreement report and
   add `--fragment-receipt <name@digest>` for binding. Clearest, costs a new flag
   and a projection-test edit.
2. Make one grammar authoritative: `name@digest` only, and have `fragment_lines`
   report on the parsed name. Fewest flags; changes the asserted output line, so
   it needs the projection test amended deliberately.
3. Leave it, document it in the arg's help. Cheapest, keeps the wart.

Prefer 2 if the receipt store is about to be written, 1 otherwise. Do not decide
this without reading PHASE-07 F-12 and F-14 first.
