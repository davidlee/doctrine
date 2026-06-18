# SL-099 Implementation Plan Rationale

## Sequencing

The 8 phases form a dependency chain that respects ADR-001 (leaf ← engine ← command,
no cycles) and minimises coupling between phases.

### Foundation first: PHASE-01 (schema)

Every other phase depends on the widened schema. `Lifespan`, `Provenance`, `RawSource`,
`Memory`, `Draft`, and `RecordArgs` are the data-model spine. The `render_memory_toml`
changes ensure new CLI flags actually land in written TOML (D3 — the silent-data-loss
fix). Template changes ensure newly recorded memories carry the new fields.

This phase is also the behaviour-preservation proof-point for backward compatibility:
existing Memory::parse tests must pass unchanged against old-format memory files.

### Leaf tier next: PHASE-02 (wikilink extractor)

`src/links.rs` is pure, has zero upward imports, and is independent of all other phases.
It can be implemented and exhaustively tested in isolation. Built early so later phases
(PHASE-03 show wikilinks, PHASE-04 inspect wikilinks, PHASE-06 BFS) can consume it.

The leaf-tier purity constraint (D1 + F1) is enforced here: all functions take
string-keyed data; Memory→string projection is the caller's responsibility.

### Read surfaces: PHASE-03 (show/retrieve) then PHASE-04 (inspect)

PHASE-03 widens the primary read surfaces (`memory show`, `memory retrieve`) and wires
the standalone `resolve-links` and `backlinks` CLI verbs. It depends on PHASE-01 (Memory
carries relations) and PHASE-02 (wikilink extraction for the wikilinks section).

PHASE-04 (inspect) is deliberately phased *after* PHASE-03, not bundled with it. The
`inspect` pre-dispatch (`MemoryRef::parse` in `run_inspect`) is architecturally distinct
from the memory-cli verb dispatch — it lives in `main.rs`'s `run_inspect`, which already
orchestrates `relation_graph` and `priority`. The `memory_inspect_view` function lives in
`memory.rs` (command tier per W1 resolution), importing downward. Separating this from
PHASE-03 keeps each phase's scope narrow and testable.

### Query machinery: PHASE-05 (lifespan filter + ageing)

Depends only on PHASE-01 (Lifespan enum, Memory.lifespan). The `effective_age` function
is pure and sits next to `days_between` in `retrieve.rs`. The sort key change is
behaviour-preserving by construction (unset lifespan = 1.0), proven by the existing
rank tests passing with zero edits post-change.

The `--lifespan` filter plugs into the existing `query()` filter cascade after
`match_scope` (design D3 / F7 position). `QueryContext` widening is the only structural
change to the query pipeline.

### Graph operations: PHASE-06 (suggested relations + --expand)

Depends on PHASE-01 (Memory, lex_doc, collect_all), PHASE-02 (links.rs for BFS edge
set), and PHASE-03 (render_show for framed block output). Bundled because both features
are graph-traversal operations over the link set — they share the same edge-building
path (wikilinks ∪ relations) and the same Memory projection patterns.

BFS expansion carefully excludes cross-entity edges (D9/F8) — non-memory targets are
skipped, keeping the graph within the memory subsystem. Cycle handling via `visited:
BTreeSet<String>` prevents infinite loops.

### Write-path hardening: PHASE-07 (validate + --allow-dirty)

Depends only on PHASE-01 (Memory carries relations, review_by, scope.paths). Three
validation checks: dangling relations (reads Memory.relations + catalog entity store),
stale verification (reads verified_sha + git), draft expiry (reads review_by + clock).
All are advisory — never writes. The `commits_touching` git primitive is designed for
multi-path queries from the start (W3) so batching can be a non-breaking follow-up.

`--allow-dirty` is a narrow change to the verify dirty-tree bail. The flag explicitly
names the tradeoff (D6/D5).

### Gate: PHASE-08 (integration + behaviour-preservation)

Runs last as a gate, not a feature phase. Verifies that all shared machinery
(`entity.rs`, `relation.rs`, `catalog/`, `retrieve.rs`, `lexical.rs`) passes unchanged
— no test was edited to accommodate new behaviour. The cross-phase e2e smoke test
exercises the full 7-objective chain: record → show → validate → find → retrieve
--expand → inspect → verify --allow-dirty.

`just gate` is the final authority: workspace-wide clippy with zero warnings.

## Why these boundaries

- **Each phase produces one testable capability increment.** No phase requires
  "imagine the rest of the system" to verify.
- **Leaf-tier work (PHASE-02) is isolated** from command/engine changes. It can be built
  and tested without touching `memory.rs` or `retrieve.rs`.
- **Read surfaces (PHASE-03, PHASE-04) are separated** because `show`/`retrieve` use
  the memory CLI dispatch while `inspect` uses the `main.rs` pre-dispatch — different
  code paths, different test strategies.
- **PHASE-05 (ageing) is before PHASE-06 (--expand)** because `--expand` uses
  `sort_default` for equal-depth ordering, which is PHASE-05 territory.
- **PHASE-07 (validate + --allow-dirty) is last among feature phases** because neither
  is a dependency of any other phase — they can be built independently once the schema
  is widened.

## Risk mitigation

- **Backward compatibility** is proven at PHASE-01 (VT-7: existing parse tests pass)
  and PHASE-08 (VA-2: existing memory files smoke test).
- **Behaviour-preservation** is gated at PHASE-05 (VT-4: existing rank tests unchanged)
  and PHASE-08 (EX-1 through EX-5: all shared machinery suites pass).
- **Data-loss prevention** (D3) is verified at PHASE-01 (VT-4 through VT-6: all new
  flags round-trip through render → parse).
- **Architectural fidelity** (ADR-001) is enforced at PHASE-02 (VA-1: no upward
  imports) and PHASE-04 (VA-1: no new imports from relation_graph into leaf tier).
