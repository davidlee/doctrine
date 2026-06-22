# Full-text search for the entity corpus

## Context

The memory system already has BM25 full-text search via the `lexical` leaf
(`src/lexical.rs` — `Bm25Ranker` implementing the generic `LexicalRanker` trait
over `LexicalCorpus` of `LexDoc`s). The BM25 engine is pure and memory-agnostic.

The entity catalog (`src/catalog/scan.rs` → `hydrate.rs`) already walks every
kind in `integrity::KINDS` (slices, ADRs, specs, RFCs, backlog items, knowledge
records, reviews, revisions, etc.) and reads metadata from their `.toml`
manifests — but it does NOT read the `.md` body prose.

Body paths follow `entity::id_path(root, kind, id, Ext::Md)` — e.g.
`.doctrine/slice/001/slice-001.md` — but nobody reads them during scan.

The gap is small: add body ingestion to the catalog scan, project entities into
`LexDoc`s (canonical id + title + body), and expose it through a `doctrine
search` command that reuses the existing BM25 ranker.

## Scope & Objectives

1. **Body ingestion into `ScannedEntity`** — add `body: Option<String>` read
   from `entity::id_path(root, kind, id, Ext::Md)` during `scan_entities`.
   Missing file → `None` (many kinds legitimately have no body).

2. **`LexDoc` projection for entities** — a pure adapter `entity_lex_doc(ent)
   → LexDoc` analogous to `retrieve::lex_doc(m: &Memory)`, packing canonical
   id + title + body (when present).

3. **`doctrine search` command** — new command-tier CLI verb:
   - `doctrine search <query>` — positional lexical query
   - `--kinds sl,adr,prd,spec,iss,imp,chr,rsk,ide` — which entity kinds to
     search (default: a sensible working set; flag is additive or
     replace-default)
   - `--format table|json` — output format
   - Reuses `Bm25Ranker` from `src/lexical.rs` — builds a `LexicalCorpus` from
     matching-entity `LexDoc`s, fits BM25, scores, returns ranked results.

4. **Default kind set** — slices, ADRs, specs (product + tech), RFCs, and all
   five backlog kinds (issue, improvement, chore, risk, idea) by default.
   Requirements and knowledge records are reasonable defaults too — TBD at
   design. Reviews, revisions, recs, and concept maps are excluded by default
   (their prose is process/graph byproducts, not primary authored content).
   `--kinds` flag replaces the default set with an explicit list.

## Non-Goals

- No precomputed/persisted index — on-demand fit per query (matches memory
  `retrieve` pattern).
- No stemming, stopword removal, or query syntax changes — reuse `lexical::tokenize`
  as-is.
- No change to memory search (`doctrine memory find`) — this is a separate
  surface.
- No change to `LexicalRanker` trait or `Bm25Ranker` — the engine is already
  generic.
- No relevance feedback or result tuning knobs (yet).
- No status filtering (display-only column). Follow-up.
- No MCP surface for agents. Follow-up.

## Summary

| File | Change |
|------|--------|
| `src/catalog/scan.rs` | Add `ScanMode` parameter + `body: Option<String>` to `ScannedEntity`; read `.md` when requested; body-read error policy (missing→None, error→diagnostic+None) |
| `src/catalog/hydrate.rs` | Carry `body` through to `CatalogEntity`; serde-skip body from JSON |
| `src/lexical.rs` | Add `tokenize_with_spans(&str) → Vec<TokenSpan>` (shared span authority); `tokenize` becomes a projection over it |
| `src/search.rs` | New module: `KindSelector`, `entity_lex_doc`, `snippet`, CLI args, BM25 orchestration |
| `src/main.rs` | Wire `Search` variant into `Commands` enum |
| `src/commands/cli.rs` | Dispatch `Search` to `search::run` |

## Risks

- **RSK-001 (disk I/O amplification):** reading every `.md` during catalog scan
  doubles the disk touches. Mitigation: `ScanMode { include_bodies: bool }`
  gates it per-caller; only search pays the cost.
- **RSK-002 (memory pressure):** holding all entity bodies in one `Vec<LexDoc>`
  during search. Mitigation: entity corpora are small (hundreds, not millions).
- **RSK-003 (template noise):** many entities start from template boilerplate
  ("## Scope & Objectives"). These tokens match every query containing "scope"
  or "context". Follow-up: strip or score-penalise high-frequency tokens.

## Verification / Closure intent

- `doctrine search "auth token"` returns ranked results from entity bodies
- `doctrine search "auth" --kind adr` limits to ADRs, excluding default kinds
- `doctrine search "auth" --with req --no sl` additive/subtractive logic
- `doctrine search "auth" --format json` emits structured output
- `doctrine search "auth" --context` shows body snippets
- `doctrine search "auth" --context --short` fails (mutually exclusive)
- `doctrine search "nonexistent"` returns zero results (score-0 suppressed)
- `just gate` green — no layering violations (new module is command-tier)
- Existing catalog tests pass unchanged (body field additive, ScanMode defaults false)
- Lexical unit tests pass unchanged (no change to the ranker)

## Follow-Ups

- Persisted index for large corpora if on-demand fit becomes slow
- Stemming/configurable tokenizer
- MCP surface for entity search (like memory has)
- Hoist group aliases from `search.rs` to `integrity.rs` (shared alongside KINDS)
- Template boilerplate detection and score-penalisation
