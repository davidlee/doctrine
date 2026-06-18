# IMP-099: Extend after/prune and overrides-adapter across entity kinds (actionability graph admission)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Historically backlog items (ISS/IMP/CHR/RSK/IDE) were the first actionable items
admitted to the **actionability graph** — the `needs`/`after` dep/sequence layer
that `backlog list` composes and renders as the `overrides:` honest-record footer.
The graph is in the process of being **extended to include slices and other
entities** beyond backlog items. The current `after`/`--prune`/`--remove` verbs
and the overrides-adapter were built for the item→item case only and do not yet
cover cross-kind edges.

Discovered at SL-105 reconcile (RV-084). SL-105 shipped `backlog after --remove`
and `backlog after --prune` for **item→item** `after` edges; that scope is
correct and complete for its slice. The shortfall is the *next* layer: cross-kind
`after` (and the ordering semantics of non-backlog entities generally).

## The shortfall

`backlog after --prune` and `--remove` (SL-105, `src/backlog.rs::run_after`) and
the overrides-adapter (`render_overrides` / `OverrideReason::Dangling`) handle
only item→item `after` edges whose target is a backlog item with a terminal
status (`resolved`/`closed`). They do not handle `after` edges targeting slices,
specs, or other entity kinds being admitted to the actionability graph:

- **`--prune` declines cross-kind terminal targets.** The probe reads the
  target TOML and drops only when `status == "resolved" || "closed"`. Slices use
  `done` (and other kinds have their own terminal vocabulary), so a dangling
  cross-kind edge is reported in the `overrides:` footer but never cleared.
- **`--remove` cannot target cross-kind refs at all.** `parse_ref`
  (`backlog.rs`) accepts only `ISS/IMP/CHR/RSK/IDE`; the `SL`/`SPEC` prefixes are
  rejected before any removal runs.
- **Slice relationships do not carry the same semantics and ordering behaviour**
  as item→item `after`. Extending the verb is not a mechanical widening — the
  ordering model for slices/entities needs its own design.

## Concrete instance (the reminder edge)

`IMP-095 → SL-095` is a **valid** cross-kind `after` edge, deliberately
**retained** as a reminder and incentive. SL-095 is `done`; the overrides footer
surfaces it as `IMP-095 → SL-095 dropped (dangling: SL-095 absent)`. It cannot be
cleared by SL-105's feature (`--prune` declines on `done`; `--remove` rejects the
`SL` prefix). It stays in the corpus until this IMP is addressed systematically.

## Out of scope for SL-105

Per RV-084 reconcile: none of this is to be solved within SL-105's lifecycle. The
SL-105 design note (`.doctrine/slice/105/design.md` §7) records the shortfall and
links here; `.doctrine/slice/105/notes.md` carries the reconcile-time detail.

## Scope (for a future slice)

- Define ordering semantics for non-backlog entities in the actionability graph
  (slices/specs/etc. do not reuse item→item `after` semantics verbatim).
- Extend `--prune`'s terminality probe across entity kinds (per-kind terminal
  status vocabularies, not just `resolved`/`closed`).
- Extend `--remove` target resolution to cross-kind refs (or route cross-kind
  edges through the appropriate entity-kind verb).
- Reconcile the overrides-adapter's `Dangling` reason with the cross-kind model
  so the `overrides:` footer reports cross-kind dangling honestly and clearably.

## Links

- SL-105 — `after` edge removal (item→item scope; this IMP's trigger)
- RV-084 — SL-105 audit; F-1 (VA-1) reconcile surfaced the shortfall
- IMP-095 → SL-095 — the retained reminder edge
- IMP-026 (triggers actionability mask), IMP-047 (trinary actionability) —
  neighbouring actionability-graph items (distinct concerns)
