# CHR-056: Retire SL-222's dead facet_write float writers

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

Three symbols in `src/facet_write.rs` are genuinely dead and carry a stale,
factually incorrect deletion marker:

| symbol | line |
|---|---|
| `toml_edit_value_as_f64` | `:39` |
| `set_facet` | `:99` |
| `apply_set` | `:308` |

Each is annotated:

```rust
#[cfg_attr(not(test), expect(dead_code, reason = "transitional facet writer; \
  migration script is the last consumer, deletes at SL-222 deletion phase"))]
```

Delete them, or — if a test-only caller crosses a module boundary — replace the
reason string with one that is true.

## Why the marker is wrong

**The premise is false.** *"migration script is the last consumer"* — the
migration scripts are `scripts/migrate_estimate_facets.py` and
`scripts/migrate_value_facets.py`, Python using stdlib `tomllib`. They never
consumed the Rust module.

**The promise was not kept.** The marker was added by SL-222 itself at PHASE-06
(commit `d8345ad5`). SL-222's PHASE-09 objective (`plan.toml:174`) committed to
*"facet_write [value]/[estimate] machinery deletes (risk/tags survive)"*, but its
exit criteria checked only the widened grep-gate (EX-1), the tripwire suite
(EX-2), and a green full suite plus baseline diff (EX-3). None verified the
deletion. SL-222 closed `done` with the functions in place.

**Nothing owns it.** RV-284's F-7 (*deliberate code residue*) covered render
arms, NF-001 allowlist minimality and `deserialize_lenient`, routed to CHR-047 —
not these. No REC row, no backlog item. This chore is the first.

## Why it is worth doing

The rest of the module is live and load-bearing: `set_facet_mixed` /
`apply_set_mixed` / `clear_facet` / `apply_clear` are unmarked and serve the
shipped `doctrine risk set` / `risk clear` via `src/commands/facet.rs:711` and
`:764`. SL-249 is about to ride that same seam for knowledge facets.

A stale *"deletes at SL-222 deletion phase"* on a module that is about to gain a
second production consumer actively misleads. It already did: SL-249's research
round read the per-function markers as module-level intent and briefly recorded
the whole seam as scheduled for deletion (R4 on `SL-249`, since dissolved). That
cost a research thread to unwind.

## Care

Verify no test-only caller crosses a module boundary before deleting — the
`cfg_attr(not(test), …)` shape means these compile in test builds.

## Related

- `SL-222` — the slice that added the marker and did not act on it.
- `SL-249` — rides the surviving half of the module; the misleading marker was
  found there. Not a blocker for it.
- `CHR-047` — SL-222's other residue chore, from RV-284 F-7. Sibling, disjoint.
