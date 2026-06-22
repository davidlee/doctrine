# Implementation Plan SL-141: Full-text search for the entity corpus

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

SL-141 adds `doctrine search <query>` — a BM25 full-text search over the entity
corpus body prose. The gap is small because the building blocks already exist:
the catalog scan (`src/catalog/scan.rs`) walks every `integrity::KINDS` entity,
the BM25 ranker (`src/lexical.rs` → `Bm25Ranker`) is generic over `LexDoc`, and
the memory retrieval path already demonstrates the pattern
(`retrieve::lex_doc` + BM25 orchestration). Three phases extract the span
authority, add body ingestion, and wire the command.

## Sequencing & Rationale

### PHASE-01: Lexical span tokenizer

The search command needs token-level byte spans for snippet extraction
(highlighting matched terms in their original context). Currently `tokenize`
returns bare `Vec<String>` — tokens without provenance. Extracting
`tokenize_with_spans` as the new authority (with `tokenize` as a thin
projection) is a zero-functional-change refactor that unblocks snippet support
while keeping the existence-proof of the existing test suite green.

**Design constraint**: `tokenize` must become a projection over
`tokenize_with_spans`, not vice versa. This means every call to `tokenize`
(including the BM25 hot path) allocates `TokenSpan` wrappers then discards
them. For the entity corpus size (hundreds of documents, ~10k tokens) this
overhead is negligible — the design values a single span authority over
micro-optimisation.

**Key risk**: the new `tokenize_with_spans` uses a manual `char_indices` loop
rather than the existing `split+filter+map`. Both must produce byte-identical
tokens for every possible input. A property test (VT-3) runs 100+ random
strings through both and asserts equivalence.

### PHASE-02: Body ingestion into catalog

The catalog scan currently reads metadata (title, status, facets, relations)
from each entity's `.toml` manifest but skips the `.md` body prose. Adding a
`ScanMode` parameter gates the extra disk I/O to callers that need bodies.

**Design decisions**:
- `ScanMode { include_bodies: bool }` with `Default` → bodyless. This keeps
  the existing `doctrine catalog scan` output clean (no body field in JSON).
- Both `scan_entities` and `scan_catalog` gain the parameter — search calls
  `scan_catalog(root, ScanMode::include_bodies())`.
- All ~22 existing call sites get `ScanMode::default()` as a mechanical update.
  Rust's type checker guarantees no missed sites.
- Body reading via `std::fs::read_to_string` on the `.md` path. Missing file →
  `None` (many kinds have no body). Permission/IO errors → `CatalogDiagnostic`
  + `None` — scan continues.
- `#[serde(skip_serializing_if = "Option::is_none")]` on `CatalogEntity.body`
  matches the existing pattern for `estimate` and `value` facets.

### PHASE-03: Search command

The entire new `src/search.rs` module plus CLI wiring. Largest phase but a
single conceptual unit.

**KindSelector** resolves `--kinds`, `--with`, `--no` into an explicit prefix
set. Case-insensitive matching against all known `integrity::KINDS` prefixes.
`--kinds` replaces defaults; `--with`/`--no` modifies defaults; `--kinds` +
`--with`/`--no`: explicit set wins, `--with`/`--no` adjust it.

**Default kind set** (resolved from design's "TBD"): slices, ADRs, RFCs,
product-specs, tech-specs, and all five backlog kinds (issue, improvement,
chore, risk, idea). Requirements and knowledge records excluded from defaults
— their body prose is sparse or ephemeral; includable via `--with req` or
`--with asm,dec,que,con`. Reviews, revisions, recs, and concept-map entities
excluded (process/graph byproducts).

**Snippet algorithm**: `tokenize_with_spans` on doc text, find first span
matching any query token, extract `context_chars` bytes around it with
ellipsis on truncation. Single-pass using the same span authority from
PHASE-01.

**JSON output**: `{ id, title, kind, prefix, score, snippet? }`.

**Table output**: `ID | Kind | Title | Score` header row, one result per row.
With `--context`, each result gets an indented snippet line below.

## Key assumptions and resolved ambiguities

| # | Issue | Resolution |
|---|-------|------------|
| A1 | `TokenSpan` uses byte or char offsets? | Byte offsets (matching `&str` indexing). |
| A2 | `tokenize` allocation overhead from projection? | Acceptable for corpus size. |
| A3 | `--kinds` (plural) or `--kind`? | `--kinds` (plural, matching design's CLI spec). |
| A4 | `--short` flag from design verification? | Removed — not in scope, only in verification artefact. |
| A5 | Requirements/knowledge in default kind set? | Excluded from defaults, includable via `--with`. |
| A6 | `ScanMode` propagation through `scan_catalog`? | Yes — both `scan_entities` and `scan_catalog` gain the param. |

## Notes

- **No persisted index.** BM25 is fitted on-demand per query (matching the
  memory `retrieve` pattern). For the current entity corpus size (hundreds,
  not millions) this is fast enough.
- **RSK-001 mitigation** (disk I/O amplification): `ScanMode::default()` is
  bodyless; only search pays the cost of reading `.md` files.
- **RSK-002** (memory pressure): accepted — entity corpora are small.
- **RSK-003** (template boilerplate noise) is deferred to a follow-up — not
  blocking this slice.
- **The `serde(skip_serializing_if)` on `CatalogEntity.body`** is consistent
  with the existing pattern for `estimate` and `value` facets.
- **All caller updates in PHASE-02 are mechanical** — Rust's type checker
  ensures no missed site. Adding `ScanMode::default()` at each call site is
  the simplest path; no wrapper functions needed.
