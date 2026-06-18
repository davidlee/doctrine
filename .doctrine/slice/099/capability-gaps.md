# Capability gaps: spec-driver memory model vs current doctrine impl

Comparison of the spec-driver memory system (`scratch/memory-spec.local.md`)
against the current doctrine implementation. Gaps are listed by severity.

## 1. Relations invisible at all CLI read surfaces

**180 `[[relation]]` rows across 100 memories exist in TOML storage.** They
are authored via `doctrine link` or hand-editing. But:

- `memory show` (text and JSON) — **no relations field**
- `memory retrieve` — **no relations**
- `doctrine inspect` — **refuses memory refs** ("not a canonical ref")
- `catalog scan` — **0 memory entities** (memories excluded from the catalog
  entity list)
- `catalog graph` — 193 memory nodes but **0 edges** (the `[[relation]]` rows
  are not consumed by the graph builder)

The authored graph exists in storage but has no read path. This is the
ship-in-a-bottle problem — data you can write but never read.

## 2. Wikilink extractor missing entirely

The spec's primary cross-reference mechanism:

| Spec says | Doctrine has |
|---|---|
| Parse `[[...]]` from body, skip code blocks | No wikilink extractor |
| `links.out` (resolved) and `links.missing` (unresolved) as **derived** metadata | No link resolution |
| `--links-to MEMORY` backlink query | No backlink command |
| `expand_link_graph()` BFS up to depth 5 | No graph traversal |
| `admin resolve links` command | No equivalent |

`record-memory` §6 tells agents to use `[[uid]]` inline ("cheaper than
relations"), which aligns with the spec's principle — but nothing resolves
them into structured edges.

**Clarification:** Extraction is cheap — a corpus-wide regex over 193
memories runs in ~0.007s. No persistence of derived links needed;
compute on-the-fly.

## 3. Verify gated on clean worktree

`doctrine memory verify` refuses a dirty working tree. An agent mid-work
cannot attest a memory they just authored. The spec's verification model
uses `verified` date + `verified_sha` for staleness computation — it doesn't
gate attestation behind a clean tree.

Compounding: a fresh `thread` is invisible to `find`/`retrieve` until
verified AND reviewed within 14 days (SL-008 D6). So recording a thread
mid-work → can't verify (dirty tree) → invisible to scope ranking. The
agent must either commit first (breaking their flow) or record as a durable
type instead.

## 4. Fields missing from schema

| Spec field | Doctrine equivalent | Gap |
|---|---|---|
| `audience` (`human`/`agent`) | — | Missing |
| `visibility` (pre-hook surfacing) | — | Missing |
| `requires_reading` (prerequisite files) | — | Missing |
| `owners` (team ownership) | — | Missing |
| `provenance.sources` (`[{kind, ref, note}]`) | Implicit git anchor only | Missing structured provenance |
| `links.out` / `links.missing` | — | Not needed — computed on-the-fly; no persistence |
| `review_by` (scheduled review date) | — | Missing |

## 5. Skills don't encourage connection-making

- `record-memory` §6 says to use inline `[[uid]]` refs — good, but never
  mentions `doctrine link` for creating formal `[[relation]]` edges
- `retrieve-memory` never mentions relations, `inspect`, backlinks, or
  graph traversal
- No `/maintaining-memory` or `/reviewing-memory` skills exist in doctrine

The result: agents build the relation graph by accident (when they happen to
run `doctrine link`) but neither skill tells them to, and no read surface
shows them the result.

## 6. Naming divergence

| Spec | Doctrine |
|---|---|
| `confidence` (low/medium/high) | `trust_level` |
| `priority.severity` + `priority.weight` | `ranking.severity` + `ranking.weight` |

## 7. Status lifecycle limited

Spec defines: `active`, `draft`, `deprecated`, `superseded`, `obsolete`,
`archived`. Doctrine currently has `active` and `draft`. No deprecation,
supersession, or archival lifecycle verbs for memories.

## Non-gaps (working as intended)

- **Catalog.** Memories are excluded from the catalog entity list but
  appear in the graph — the user confirms this is fine for its only consumer
  (the web explorer).
- **`[[relation]]` write path.** Memory labels are free-form strings
  (`CatalogEdgeLabel::Raw`), distinct from the vocabulary-bound
  `RelationLabel` for numbered entities. This is documented in
  `mem.pattern.link.memory-label-fork` and is intentional.
- **Storage model.** TOML + MD, no database — matches the spec-driver
  file-memory model. The forgettable compatibility contract
  (`memory-contract.local.md`) describes a future backend, not a current gap.
