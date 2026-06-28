# SL-173 — design: `backlog list --after / --needs` edge filter

## Current → target behaviour

`doctrine backlog list` filters rows by substring/regex/status/tag/kind and
orders them with `--by` (`sequence` default, `id`). The `needs`/`after`
dependency-sequence edges are queryable only indirectly (authoring via `backlog
needs`/`backlog after`; `relation list` covers tier-1 `[[relation]]` edges, which
these are not). The question "which items declare an edge pointing at X?" is
inexpressible.

**Target.** Two new repeatable row filters on `backlog list`:

- `--after <REF>` — retain items with an `after` edge whose `to` matches any
  given ref.
- `--needs <REF>` — retain items whose `needs` axis contains any given ref.

OR within a flag (repeatable, mirroring `--tag`); AND across axes (with each
other and with substr/regex/status/tag/kind, mirroring `--kind`). The terminal
hide-set (`resolved`/`closed` hidden by default) is unchanged — combine with
`--all` / `-s resolved,closed` to see everything. Table, JSON, and `--columns`
render as usual; only row membership changes.

## Architecture fit (the seam)

`--kind` is **not** a shared `listing` axis — it is a backlog-local
`items.retain(|i| kind.is_none_or(…))` applied at `src/backlog.rs` immediately
after `listing::retain` runs the shared substr/regex/status/tag axes plus the
hide-set. `--after`/`--needs` ride this exact pattern: two further local
`retain`s, backlog-scoped (non-goal: no promotion to a shared axis, no other
kinds).

`BacklogItem.relationships` is already materialised in the read corpus
(`needs: Vec<String>`, `after: Vec<AfterEdge { to, rank }>`). The filter is pure
over the in-memory corpus — **no new I/O** (ADR-001 layering holds: command
layer, no leakage; pure/imperative split: the predicate is pure).

Ordering is unaffected. `compose` borrows the **full** non-terminal corpus
before `retain` moves it; the new filters change membership only, then surviving
rows sort by the precomputed sequence positions exactly as today.

## Match semantics — normalized (D1)

Edges are stored **verbatim** at authoring time (`dep_seq::append` does no
canonicalisation), so a stored `to`/`needs` ref is whatever the author typed.
Matching is **normalized**, not raw-string:

```rust
/// Canonicalise a ref for edge-filter comparison: parse to (kind, id) and
/// re-render. Pure (no disk). An unparseable authored ref falls back to its
/// verbatim string so it stays findable.
fn norm_ref(r: &str) -> String {
    match crate::integrity::parse_canonical_ref(&r.to_uppercase()) {
        Ok((k, id)) => listing::canonical_id(k.kind.prefix, id),
        Err(_) => r.to_string(),
    }
}
```

`parse_canonical_ref` is cross-kind (an edge may point at any kind — e.g. `needs
SL-169`, not just backlog kinds), so backlog-local `parse_ref` (ISS/IMP/CHR/RSK/
IDE only) is **wrong** here. Uppercasing the input first absorbs case
(`kind_by_prefix` is case-sensitive); `canonical_id`'s `{id:03}` absorbs padding.
Thus `--after imp-0194`, `IMP-194`, `IMP-0194` all match a stored `IMP-194`.

This honors the scope's real intent — **no existence resolution**, so a
dangling/deleted ref still matches — while dropping the brittle raw-string
equality. The two are separable: normalization does no disk access.

## Predicate

Item passes `--after` iff its `after` edges and the supplied refs share at least
one normalized value:

```text
after_ok  = args.after.is_empty()
          || item.after.iter().any(|e| afterset.contains(&norm_ref(&e.to)))
needs_ok  = args.needs.is_empty()
          || item.needs.iter().any(|n| needsset.contains(&norm_ref(n)))
keep      = after_ok && needs_ok   // AND, composed atop the shared retain ∩ --kind
```

`afterset`/`needsset` are the normalized query refs, built once. An empty flag
slice is a no-op (the axis imposes no constraint).

## Code impact (= `design-target` touch-set)

`src/backlog.rs` only:

| Site | Change |
|---|---|
| `BacklogCommand::List` enum | add `#[arg(long)] after: Vec<String>` and `#[arg(long)] needs: Vec<String>` (help mirrors `--tag`) |
| dispatch arm (`BacklogCommand::List => run_list(…)`) | thread `after`, `needs` into `run_list` (params, not `ListArgs` — same as `kind`) |
| `run_list` / `list_rows` signatures | add `after: &[String]`, `needs: &[String]` |
| `list_rows` filter step | insert the two `retain`s right after the `--kind` retain, **before** the `any_tagged` computation |
| new `norm_ref` helper | pure normalizer (above) |
| `list_rows` tests | extend existing unit tests |

No change to `src/listing.rs`, `src/dep_seq.rs`, JSON shape, or ordering.

## Verification impact

Unit tests on `list_rows` (the pure compute half):

- `--after IMP-194 --all` retains only items with that `after` edge.
- **Normalization**: `--after imp-0194` matches a stored `IMP-194`.
- Cross-kind: `--needs SL-169` matches a `needs` ref of `SL-169`.
- `--after` AND `--status` AND-compose.
- `--after` AND `--needs` — item must carry both.
- Unparseable authored ref matches verbatim (raw fallback).
- `--by sequence` order intact over the filtered set.
- `--json` and `--columns` render correctly on the filtered set.

CLI-level checks per the slice's Verification section.

## Decisions / open questions

- **D1 — normalized match** (locked): parse-and-compare `(kind, id)` via
  `parse_canonical_ref`, verbatim fallback. No existence check; dangling still
  matches.
- No open questions. Repeatable/OR/AND-compose/hide-set settled by scope.
