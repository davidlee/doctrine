# SL-092: inspect sort + scan robustness — RSK-007 + IMP-036

## Context

Two deferred findings from the SL-046 code review, both in the `inspect`/`scan`
pipeline:

- **RSK-007** — the inbound render sort in `relation_graph::inspect_from` is
  lexical on canonical-ref strings (`"SL-1000" < "SL-999"`). Zero-pad is min-3,
  not fixed-width; the cliff is at id ≥ 1000. The permutation-invariance test
  only exercises single-digit ids — false confidence.
- **IMP-036** — `scan_entities` aborts the whole corpus scan on any one
  malformed sibling's `outbound_for` `?` — a single corrupt TOML blocks every
  `inspect`/`survey`/`next`/`backlog list` invocation. The scan should degrade
  gracefully per-entity.

Both are small, unblocked, and touch the same module neighbourhood. Bundling
avoids two micro-slices on the same pipeline.

## Scope

### RSK-007: numeric sort for inbound refs

- In `inspect_from` (and any other sort site that collates canonical refs):
  sort by `(prefix, numeric_id)` instead of lexical string sort on
  `EntityKey::canonical()`.
- Extend `inbound_render_is_permutation_invariant` to exercise id ≥ 1000
  (seeding e.g. SL-998, SL-999, SL-1000, SL-1001).

### IMP-036: graceful degradation on malformed sibling

- In `scan_entities`: replace the `outbound_for(root, kref.kind, id)?` with
  a `match` that skips the malformed entity, accumulates a diagnostic, and
  continues. The queried entity's own parse failure remains a hard error
  downstream (the F6 existence gate already handles it).
- Surface the skip as a diagnostic (stderr note) at the command layer. A
  `--strict` flag that restores fail-fast is deferred (out of scope — the
  IMP-036 body flags it but does not require it).

### Non-scope

- Read-amplification reduction (IMP-036 body "adjacent" issue: second scan pass,
  all-corpus dangler map, third queried-entity parse). This is a separate
  improvement with its own design footprint.
- `--strict` flag. Deferred.
- Any change to `priority::graph::build` beyond the mechanical effect of the
  `scan_entities` change (it already consumes `ScannedEntity` — a skipped entity
  simply contributes no node/edges, which is the desired behaviour).
- `build_relation_graph_from` — it takes a pre-scanned slice (`&[ScannedEntity]`)
  and does NOT call `outbound_for`; it is already safe.

## Affected surface

| Path | Change |
|---|---|
| `src/relation_graph.rs` `inspect_from` (line ~581) | `srcs.sort()` → sort by `(prefix, id)` |
| `src/relation_graph.rs` test `inbound_render_is_permutation_invariant` | Extend past id 999 |
| `src/catalog/scan.rs` `scan_entities` (line ~163) | `?` → `match` with skip+diagnostic |
| `src/main.rs` (command layer) | Report accumulated diagnostics to stderr |

## Risks

- **Low** — both changes are mechanical; no new dependencies, no schema change.
- The `scan_entities` change affects every consumer. All existing suites must
  remain green unchanged (the behaviour-preservation gate). A skipped entity
  contributes nothing to the scan, which is already how absent entities behave.

## Verification

- Existing test suite stays green — no golden changes.
- New unit test: `scan_entities` with one malformed sibling returns the remaining
  entities + diagnostics.
- New unit test: inbound sort with mixed-prefix, ≥1000 ids yields numeric order.
- `cargo clippy` zero warnings.
- `just gate` green.
