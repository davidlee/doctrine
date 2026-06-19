# IDE-013: Estimate/value change history (time-series of facet edits)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced during SL-118 design (estimate/value authoring CLI verb). Desire: a
time-series record of how an entity's `[estimate]`/`[value]` facet changed over
time, so estimate drift is observable rather than overwritten.

## Why not in SL-118

The tempting shortcut — have the authoring writer stash history in an unread
permissive-serde field (`[[estimate.history]]`, absorbed by the `_extra`
flatten / NF-003 forward-compat seam) — writes **data nothing reads, governed by
no REQ**. That is the unspec'd-residue debt pattern SL-104 is currently paying
down (confidence legitimization). It also locks a wire shape with zero design.

SL-118 ships history-**ready**, not history-bearing: its writer is edit-preserving
(`toml_edit` insert touches only `lower`/`upper`/`value`), so a future history
sub-key survives every `set`. A verification item in SL-118 pins this (a `set`
over an `[estimate]` table carrying an unknown sub-key preserves it).

## Open questions (for a future design)

- **Governance.** Wants a REQ in SPEC-020 (meaning, reader, retention) — routed
  like the confidence legitimization (Revision-folded), not smuggled through
  `_extra`.
- **Granularity.** No clock in the pure layer — only an injected date. Date-only
  stamps mean multiple `set`s in one day collapse or stack; is that acceptable?
- **Reader.** Who consumes it — the `show` path (IMP-112), a dedicated `history`
  verb, the graph projection?
- **Wire shape.** `[[estimate.history]]` array-of-tables `{ lower, upper, at }`?
  Separate `[value]` trail? One shared `[history]` table keyed by facet?
- **`clear` semantics.** `clear` removes the whole facet table — history with it.
  If history exists, that is a retention decision (purge vs preserve trail).

## Links

- Surfaced by [[SL-118]] (estimate/value authoring CLI verb).
- Forward-compat mechanism: SPEC-020 NF-003 (`_extra` flatten, `e19` /
  `custom_deserialize_unknown_keys`).
- Sibling debt precedent: SL-104 confidence legitimization.
