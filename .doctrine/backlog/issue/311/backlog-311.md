# ISS-311: Legacy design oracle sweeps managed output

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

`tests/e2e_design_legacy_corpus.rs` is red on a clean tree:

```
import refuses a design this repo authored:
  [(".doctrine/slice/244/design.md", UnheadedPreamble { line: 1 })]
test result: FAILED. 67 passed; 1 failed
```

Reproduce with `cargo test -p doctrine --test e2e_design_legacy_corpus` — the
test reads the live working tree (`common::repo_root()` resolves at runtime,
CHR-014), so no rebuild is needed to change the outcome. It surfaced through
`doctrine check gate` → `test-all`.

## Diagnosis — the selector, not either reader

Both readers are correct, and their refusals are mutually exclusive by
construction:

| reader | input class | refuses the other class as |
|---|---|---|
| `design_run::document::parse` | managed, marker-led | `MarkerFreeAddition` (`document.rs:300`) |
| `design_run::legacy::read` | legacy, unmarked prose | `UnheadedPreamble` (`legacy.rs:137`) |

The emitter is sound too: `document::render` (`document.rs:224`) frames every
section with `MARKER_OPEN` at column 0, so a materialised document's line 1 is
`<!-- doctrine:section sec-1 -->` by construction.

The defect is that the oracle uses **path** — every `.doctrine/slice/*/design.md`
— as a proxy for **document class**, which is a property of content (a marker
line at line 1). That proxy held when SL-233 PHASE-11 wrote the test and was
falsified by `doctrine design materialise` in the same slice: PHASE-11 shipped
the reader whose oracle is *every authored design*, alongside the writer whose
output makes *every authored design is legacy prose* false. SL-244 is the first
managed document in the corpus; it is not defective, it is being read by the
wrong reader.

## Second-order: the floor rots

`CORPUS_FLOOR: usize = 200` guards against a vacuous pass over ~228 designs. It
is stated over the **legacy** count. Import (legacy → managed) is a capability
SL-233 built, so that count decreases monotonically as designs migrate, and a
correct oracle will eventually red with *"a pass would be vacuous"* for a reason
that has nothing to do with the reader. The floor belongs on total designs swept
across both classes.

## Proposed fix — partition, do not relax

Partition the corpus on whether line 1 is a marker line
(`document::marker`, `document.rs:160`), then:

- legacy half → `legacy::read` plus today's asserts, unchanged;
- managed half → `document::parse(text, None)` (no run state needed) and a
  `render ∘ parse == identity` round-trip;
- floor on the union.

This closes a coverage gap rather than merely un-redding: managed corpus
documents currently have **no** corpus-level oracle — only the inline synthetic
round-trip at `document.rs:490`. Partitioning makes the oracle total over the
corpus instead of single-class.

Relaxing the preamble rule is the wrong move: it would contradict `legacy::read`'s
own contract, and the test's `text.ends_with(&body)` head-blank assert would fail
anyway once marker bytes stopped landing in a region.

## Not a divergence: the missing `#` H1

A managed document carries no `#` title — the title is structured data
(`slice-244.toml` `title`), and every section body is title-bearing at its own
level (`## Governing context`). The test's neighbouring assert is
`!regions.is_empty()`, whose comment says *"a real design has at least its `#`
title"* but which only requires *some* heading region. No assert depends on the
H1. One divergence, not two.

## Provenance

Found while closing SL-241 — the red was initially mistaken for SL-241's own,
which it never was (SL-241's `design.md` imports clean; the sole refusal names
`slice/244/design.md`).
