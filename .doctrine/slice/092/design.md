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
normal output.

`scan_catalog` already accumulates diagnostics in its own `Vec` and returns
them in `Catalog.diagnostics`. The catalog's diagnostics flow through existing
render paths for survey, next, and other catalog consumers — no new stderr
printing needed there.

`priority::graph::build` and the test-only `inspect` wrapper pass
`&mut vec![]` — diagnostics are discarded (priority doesn't display per-entity
scan errors).

### D4 — Two match arms, not one

`status_and_title_for` and `outbound_for` are separate reads with different
failure semantics (meta parse vs relation-block parse). Merging them into one
match would couple distinct failure modes and produce a less specific error
message. Each gets its own `match` arm with a targeted diagnostic message.

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
| `src/catalog/scan.rs` tests | Two new tests: skip-malformed, all-malformed-empty |
| `src/catalog/hydrate.rs` `scan_catalog` | Pass `&mut diagnostics` to `scan_entities`; ~3 lines |
| `src/main.rs` `run_inspect` | Pass collector, print non-empty diags to stderr; ~5 lines |
| `src/priority.rs` `build` | Pass `&mut vec![]`; ~1 line |
| `src/relation_graph.rs:523` `inspect` wrapper | Pass `&mut vec![]`; ~1 line |

## Verification

- `inbound_render_is_permutation_invariant` extended: seed SL-998, SL-999,
  SL-1000, SL-1001 as supersedors → assert order SL-998, SL-999, SL-1000,
  SL-1001.
- New test: `scan_entities` with one malformed TOML returns remaining entities
  + one Error diagnostic; the good entity's fields are intact.
- New test: `scan_entities` with all-malformed siblings returns empty
  `Vec` + N diagnostics; no panic.
- New test: mixed-validity — two good, one bad → two entities returned, one
  diagnostic.
- Existing suite stays green — behaviour-preserving for well-formed corpus
  (the gate).
- `cargo clippy` zero warnings.
- `just gate` green.

## Remaining open questions

None. Both changes are mechanical, ride existing infrastructure, and are
scoped tightly to the documented call sites and test surfaces.
