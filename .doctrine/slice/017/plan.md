# Implementation Plan SL-017: Pluggable lexical scorer: trait + BM25 backend for memory retrieval

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Four phases, ordered by **risk then dependency**. The design (`design.md`) is
locked through two adversarial rounds; its two gating unknowns are a crate-API
question and a determinism question, both about the `bm25` backend. So the plan
front-loads a probe phase that *resolves those unknowns against reality before a
single line of production scoring is written*, then builds the pure seam, then
the BM25 backend against the confirmed recipe, then wires BM25 in as the hard
default while proving the old behaviour preserved.

The cut points follow the design's own layering:

- **PHASE-01** isolates the external risk (the `bm25` crate) — a keeper
  characterization test, not a throwaway spike. It owns the dependency and the
  recipe; if the recipe is unattainable it ends in `/consult`, not a workaround.
- **PHASE-02** is the entire bm25-*independent* leaf: the moved lexer, the trait,
  `OverlapRanker`, `quantize`. It can be built and fully tested with no reference
  to bm25 at all, and it carries the behaviour-preservation parity proof.
- **PHASE-03** is the only phase that touches bm25 in production code, kept small
  and behind the trait the previous phase already proved.
- **PHASE-04** is the integration: the `retrieve` restructure, the default flip,
  and the whole-system guarantees (behaviour-preservation, determinism, storage
  model).

## Sequencing & Rationale

**Why probe first (PHASE-01).** The design mandated it, and for a concrete
reason: the original draft's fit path (`with_fit_to_corpus(Language, …)`) turned
out to be feature-gated behind `default_tokenizer`, which `--no-default-features`
removes — discovered only by reading the crate source during the adversarial
pass. Two further unknowns survive into implementation: whether the
custom-tokenizer + manual-`avgdl` path is actually reachable, and whether bm25's
internal `HashMap` use lets a per-document score *value* drift across process
runs. Both would invalidate downstream phases if discovered late. A probe that
confirms the recipe, the `avgdl`/`doc_len` equivalence (A3), and cross-process
score stability (OQ-5/R7) converts three "we believe" assumptions into tested
facts — or stops the slice cleanly before sunk cost. The probe deliberately uses
a trivial local tokenizer so it characterizes *bm25*, independent of the doctrine
lexer that lands next.

**Why the pure leaf second (PHASE-02).** It has zero dependency on the probe's
outcome — the trait, the overlap scorer, and the quantizer are pure and
bm25-free. Building it here means the seam (`LexicalRanker`), the A1 total/
positional contract, and the behaviour-preservation parity (`OverlapRanker`
byte-identical to the still-present `lexical_score`) are all proven before any
BM25 code exists. Moving `tokenize()` here — after enumerating its callers (A6) —
keeps the leaf the single home of the lexer, so PHASE-03's custom bm25 tokenizer
can delegate to it without a second regime.

**Why BM25 third (PHASE-03).** It depends on *both* predecessors: the PHASE-01
recipe (how to drive the crate) and the PHASE-02 trait (what shape to implement).
Keeping it its own phase confines the only production use of an external IR crate
to one small, well-tested unit, behind an interface that already has a proven
second implementation. The corpus-relative nature (fit over all active, score
survivors) and the `avgdl` equivalence live here, exercised with the real lexer.

**Why integration last (PHASE-04).** Only now does the system change behaviour.
The `query()` restructure (active/survivors split, `LexDoc` construction, the
`&dyn LexicalRanker` parameter, positional assembly) and the default flip are
mechanical *given* the two ranked implementations. The phase's weight is in its
guarantees, not its code: the behaviour-preservation gate (existing suites green
via `OverlapRanker`), the cross-process determinism VT (the empirical answer to
OQ-5, with the coarsen-`LEX_SCALE` fallback if it fails), exact-key dominance
under BM25, and the storage-model proof (no new field, no float payload, Memory
serialization unchanged). Deleting `lexical_score` is safe here because its
replacement and its parity proof both already exist.

## Notes

- **Stop conditions are real.** PHASE-01 EX-5 and R7's rung 5 both end in
  `/consult` rather than a silent broadening of features or an unproven scale.
  Determinism wins over lexical resolution wherever they conflict.
- **Layering invariant across all phases:** `lexical` is a pure leaf importing
  neither `retrieve` nor `memory` (ADR-001); `retrieve` adapts `Memory` into
  `LexDoc`. The dependency arrow never reverses.
- **No persisted state at any phase.** Lexical scores live only in the transient
  `Candidate`; SL-017 adds zero fields to `Memory`, the payload, or any on-disk
  artifact, and introduces no index/cache (the `Indexed` corpus variant is a
  named follow-up, not SL-017 work).
