# Implementation Plan SL-202: Memory body wikilinks as catalog edges

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases turn design SL-202 into code across two files (`src/memory.rs`,
`src/catalog/hydrate.rs`). PHASE-01 lands the body on the record (impure I/O,
isolated). PHASE-02 consumes it for edges (pure projection, the risk area).

## Sequencing & Rationale

**Why two phases, in this order.** The change has a natural producer/consumer
seam. PHASE-02's body-wikilink pass needs `MemoryCatalogRecord.body` to exist; if
both landed in one phase the impure read and the pure edge logic would tangle in a
single red/green cycle. Splitting keeps each phase independently green:

- **PHASE-01** adds the field and the sibling `memory.md` read in
  `read_catalog_record` — the single impure I/O site (one prod caller, `scan.rs`).
  It produces **no edge-behaviour change** (the field is read but unconsumed), and
  is proven in isolation by VT-1 (populated body; missing `.md` → `None`). This
  de-risks PHASE-02 by settling the I/O + tolerance contract first.

- **PHASE-02** is the substance: the body-wikilink pass in `from_scanned`, reusing
  `extract_wikilinks`/`classify_target`/`EdgeTarget` verbatim (no parallel
  implementation). The three hard points are (a) **dedup** on the resolved
  `CatalogKey` via a per-record `seen` set the TOML pass feeds, (b) **diagnostic
  parity** — unresolved links warn like TOML relations, and (c) **behaviour
  preservation** — the TOML pass is unchanged bar an additive `seen.insert`, held
  by the existing edge/backlink suites (VA-1).

**Fixture prerequisite (EN-2).** `seed_memory` (scan.rs tests) writes only
`memory.toml` — its `body` param is TOML content, misnamed. PHASE-02 must first
extend/add a helper that also writes a `memory.md` body, else VT-1..4 cannot seed
body wikilinks. This is test infrastructure, not a design change (A5 from the
design's adversarial pass).

**TDD shape.** Both phases are red/green/refactor. PHASE-01: write VT-1 red
(no body field) → add field + read → green. PHASE-02: helper first, then each VT
red→green, dedup and diagnostic tests driving the loop structure.

## Notes

- Boundary (design A4): body wikilinks are memory-target-only — the extractor
  regex gates `mem.`/`mem_`, so `classify_target`'s canonical-ref branch is
  unreachable from this pass. No test asserts a body→numbered edge; none can exist.
- `src/catalog/scan.rs` is touched only for the PHASE-02 test helper, not
  production code — production edits are confined to the two design-target files.
