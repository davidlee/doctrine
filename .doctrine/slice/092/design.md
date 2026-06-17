# SL-092: Design — inspect sort + scan robustness

## Design decisions

### D1 — Sort inbound refs by `EntityKey::Ord` (numeric id within prefix)

**RSK-007.** `inspect_from` collects inbound source references as `Vec<String>`
via `EntityKey::canonical()`, then sorts lexicographically. At id ≥ 1000,
`"SL-1000" < "SL-999"` because `'1' < '9'`. The zero-pad is min-3, not
fixed-width.

`EntityKey` already derives `Ord` — compares `prefix: &str` lexicographically,
then `id: u32` numerically. This is the correct sort for cross-kind ordering
and numeric within a kind. The fix collects `Vec<EntityKey>`, sorts that, then
maps to canonical strings.

No new sort function, no new types, no `EntityKey` repr change. The derived
`Ord` is stable and already tested implicitly by every surface that sorts
`EntityKey` in a `BTreeSet` or `BTreeMap`.

Cross-prefix ordering (e.g. ADR-001 before SL-001 because `'A' < 'S'`) is
exercised implicitly by `scan_ids` producing a `BTreeSet<EntityKey>` and by
projection/resolution tests — the RSK-007 finding is specifically about the
numeric-within-prefix cliff at id ≥ 1000. The PHASE-01 test extension therefore
covers same-prefix ordering only; a cross-prefix inbound-sort assertion would
be redundant with existing `BTreeSet<EntityKey>` coverage.

**Sort site:** single change in `inspect_from` (`relation_graph.rs`), the
inbound render block. No other site sorts canonical-ref strings bound for
display — `dep_seq_for` produces per-entity edges consumed by the priority
graph which sorts internally; `build_relation_graph_from` mints in scan order;
render sections are label-grouped only.

### D2 — Graceful degradation via `&mut Vec<CatalogDiagnostic>` parameter

**IMP-036.** `scan_entities` currently uses `?` on two per-entity reads
(`status_and_title_for`, `outbound_for`). A malformed TOML on any sibling
entity aborts the entire corpus walk, blocking inspect, survey, next, backlog
list, explain, blockers.

The fix adds `diagnostics: &mut Vec<CatalogDiagnostic>` to `scan_entities`.
On either read failure, a `CatalogDiagnostic` with `Severity::Error` is pushed,
and the loop continues to the next id. A skipped entity contributes nothing to
the returned `Vec<ScannedEntity>` — consumers never see it. This is identical
behaviour to an entity that was never minted (absent dir).

The parameter channel matches `scan_memory_entities`'s existing precedent
(`src/catalog/scan.rs:240`).

**Failures that remain fatal (return `Err`):**
- `entity::scan_ids` fails to read a top-level kind directory (e.g. permission
denied on `.doctrine/slice/`).
- Any error outside the per-entity `for id in ids` loop (KINDS table walk,
initial setup).

Only the two per-entity reads (`status_and_title_for`, `outbound_for`) are
softened to `match` + skip.

**The queried entity's own parse failure remains hard.** The F6 existence gate
(`require_minted`) in `inspect_from` / `render_from` checks whether the
queried entity's key is in the `Projection<EntityKey>` before reading it. If
`scan_entities` skipped it, the gate errors `"KIND-NNN: no such entity"` — the
same message as a never-minted id. The scope explicitly scopes queried-entity
failure as a hard error (out of scope).

### D3 — Command-layer surface: diagnostics to stderr

The `run_inspect` handler in `main.rs` is the single direct `scan_entities`
consumer that surfaces to a user. It passes a throwaway `Vec` to collect
diagnostics, and after scan prints any non-empty diagnostics to stderr before
normal output. Format: one `eprintln!` per diagnostic, `"{id}: {message}"`,
matching the `validate` findings style. The stdout output (human table / JSON)
is unchanged — byte-identical for a well-formed corpus.

`scan_catalog` already accumulates diagnostics in its own `Vec` and returns
them in `Catalog.diagnostics`. Entity-scan errors and memory-scan errors land
in the same `Vec`, with entity-scan errors ordered first (KINDS-table order).
The catalog's diagnostics flow through existing render paths for survey, next,
and other catalog consumers — no new stderr printing needed there.

`priority::graph::build` and the test-only `inspect` wrapper pass
`&mut vec![]` — diagnostics are discarded (priority doesn't display per-entity
scan errors).

### D4 — Two match arms, not one

`status_and_title_for` and `outbound_for` are separate reads with different
failure semantics (meta parse vs relation-block parse). Merging them into one
match would couple distinct failure modes and produce a less specific error
message. Each gets its own `match` arm with a targeted diagnostic message.

### D5 — Dangler amplification: skipped-entity references become danglers

When a sibling entity is skipped (malformed TOML), any other entity that
references it via an outbound edge will display that reference as a **dangler**
in `inspect` output — indistinguishable from a free-text target that was never
minted. This is an accepted trade-off: the stderr diagnostic flags the
malformed entity, and the dangling reference is a visibility cue rather than a
silent omission. The alternative (partial entity with meta but no edges, or
propagating a degraded-marker through the projection) would require a larger
change to `ScannedEntity` and every consumer — out of scope for IMP-036. The
`--strict` flag (deferred) would be the natural place to restore fail-fast
behaviour for consumers that reject this trade-off.

## Current behaviour vs target behaviour

| Surface | Current | Target |
|---|---|---|
| `inspect SL-001` with inbound from SL-999, SL-1000 | SL-1000 listed before SL-999 | SL-999 before SL-1000 (numeric order) |
| `scan_entities` with one malformed sibling TOML | `Err(...)` — entire scan aborts | Returns remaining entities + one Error diag |
| `inspect SL-001` with one malformed sibling | Error — scan aborted | Succeeds; stderr note about skipped sibling |
| `survey` with one malformed sibling | Error — scan aborted | Succeeds; diagnostic in catalog output |
| Queried entity itself is malformed | Error (unchanged) | Error (unchanged — F6 gate) |

## Code impact

| Path | Change |
|---|---|
| `src/relation_graph.rs` `inspect_from` inbound block | Sort `Vec<EntityKey>` before `.map(canonical)`; ~5 lines |
| `src/relation_graph.rs` `inbound_render_is_permutation_invariant` test | Seed ids ≥ 1000; ~8 lines |
| `src/catalog/scan.rs` `scan_entities` signature | Add `diagnostics: &mut Vec<CatalogDiagnostic>` param |
| `src/catalog/scan.rs` `scan_entities` loop body | Two `?` → `match` with `continue` + diagnostic; ~25 lines |
| `src/catalog/scan.rs` tests (3 existing + new) | Existing calls (lines 317, 336, 392) pass `&mut vec![]`; new tests: skip-malformed (both failure channels), all-malformed, mixed-validity |
| `src/catalog/hydrate.rs` `scan_catalog` | Pass `&mut diagnostics` to `scan_entities`; ~3 lines |
| `src/main.rs` `run_inspect` | Pass collector, print non-empty diags to stderr; ~5 lines |
| `src/priority/graph.rs` `build` (line 119) | Pass `&mut vec![]`; ~1 line |
| `src/priority/graph.rs` test (line 502) | Pass `&mut vec![]`; ~1 line |
| `src/relation_graph.rs:523` `inspect` wrapper (test-only) | Pass `&mut vec![]`; ~1 line |
| `src/relation_graph.rs` test (line 1076) | Pass `&mut vec![]`; ~1 line |

## Verification

- `inbound_render_is_permutation_invariant` extended: seed SL-998, SL-999,
  SL-1000, SL-1001 as supersedors of SL-001, planted out-of-order on disk →
  assert inbound order `["SL-0998", "SL-0999", "SL-1000", "SL-1001"]`
  (numeric, not lexical). Cross-prefix ordering is covered implicitly by
  `BTreeSet<EntityKey>` construction in scan-order and projection tests.
- New test: `scan_entities` with `status_and_title_for` failure → remaining
  entities + one Error diagnostic; assert `severity == Severity::Error`,
  `entity_key` matches the skipped entity, and `file` is populated.
- New test: `scan_entities` with `outbound_for` failure → entity skipped +
  one Error diagnostic; diagnostic message differs from the meta-parse case.
- New test: `scan_entities` with all-malformed siblings returns empty
  `Vec` + N diagnostics; no panic.
- New test: mixed-validity — two good, one bad → two entities returned, one
  diagnostic.
- New integration test: `scan_catalog` with one malformed entity returns
  remaining entities + diagnostic propagated through `Catalog.diagnostics`;
  `severity`, `entity_key`, and `file` survive the round-trip.
- Existing suite stays green — behaviour-preserving for well-formed corpus
  (the gate).
- `cargo clippy` zero warnings.
- `just gate` green.

## Remaining open questions

None. Both changes are mechanical, ride existing infrastructure, and are
scoped tightly to the documented call sites and test surfaces.
