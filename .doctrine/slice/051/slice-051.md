# retire backlog order; fold ordering into list as default-on comparator

## Context

IMP-028. `backlog order` and `backlog list` are divergent surfaces over the same
spine. `list` carries the full grammar — `--kind/--filter/--regexp/--status/
--tag/--all/--format/--json/--columns` (the `listing.rs` shared column model).
`order` carries only `--path`, and to see *ordered* output you must abandon every
filter/format/column the user already knows from `list`. Discovered inspecting a
soft `after` cycle (RSK-001 ⟷ ISS-003): `list` never shows sequence, `order`
does — but you cannot filter or reshape the ordered view.

The card's accepted framing rejects teaching `order` the list grammar (a whole
second verb for "same rows, different order" is grammar duplication). Instead:
**retire `order`, fold ordering into `list`.** One verb, one surface.

Refined in preflight to **default-on**: composed `after`/`needs` order is the
*default* row sequence for `list`, with an opt-out to restore id-sort — not an
opt-in `--order` flag. Default-on is the stronger end-state but forces a cleaner
decomposition (below): ordering becomes a pure comparator, membership stays the
filter's job, and `order`'s fail/hide behaviour cannot be inherited wholesale.

## Scope & Objectives

The decomposition default-on forces — ordering is a *sort*, not a *view*:

1. **Ordering = pure comparator** over the rows `retain` already kept. Membership
   is unchanged from today's `list`. Nodes outside the composed chain (blocked,
   edge-less, or filtered) take a defined tail position (the `(kind.ordinal, id)`
   fallback). The non-terminal projection `order` used for *membership* is dropped
   here — retained only to compute the diagnostic (objective 3).
2. **Graceful-degrade on a `needs` cycle.** Default `list` stays total: on a
   dependency cycle it falls back to id-sort plus a warning — never the hard-error
   / empty-table that `order` emits today. The everyday survey verb must not break
   on a cyclic graph.
3. **Diagnostic rides as a conditional footer.** The `overrides:` honest-record /
   dropped-edge block prints only when non-empty — absent on a clean default
   survey, never a column.
4. **Opt-out flag** restores id-sort (spelling settled at design — `--by id` /
   `--no-order`).
5. **Retire `BacklogCommand::Order` / `run_order`**; migrate its golden onto
   `list`.
6. **Composition for free** — `--kind/--status/--filter/--tag/--columns/--format/
   --json` now compose with ordering (the merge's whole point).

## Non-Goals

- No redesign of `listing.rs` — `list` *adopts* ordering as a comparator slot.
- No change to the backlog data model, relation schema, or storage tiers
  (`needs`/`after` edges unchanged).
- No fix to RSK-005 (the `backlog_order` adapter's duplicate-`ItemId` bimap
  corruption) — adjacent (same adapter call site), but a separate card.
- No PRD-009 amendment: the spec binds the ordering *capability* (FR-010 /
  `REQ-097`, the next-work views), not the literal verb name `order`. Retiring the
  verb keeps the requirement satisfied — a reconcile note, not a spec change.
- No deprecation cadence — clean cut (backlog is internal tooling).
- Not folded into SL-049 (scope-frozen to IMP-017 + ISS-004).

## Affected Surface

- `src/backlog.rs` — fold `order_rows` (1560) into the `list_rows` (900) path as a
  comparator branch; remove `run_order` (1595); decouple the non-terminal
  projection from membership (it survives only to feed the diagnostic).
- `src/main.rs` — remove the `Order` variant (887) + dispatch (`main.rs:2222`) +
  the access-classify arm (`BacklogCommand::Order` at 1685) + help; add the `--by`
  opt-out to the backlog `List` clap variant (NOT the shared `CommonListArgs`/
  `ListArgs` — ordering is backlog-local); thread it into `run_list`.
- `src/backlog_order.rs` — the cordage `BacklogOrder` adapter is reused unchanged.
- `src/listing.rs` — read-only and **unchanged** (column model + render). The
  comparator stays in `backlog.rs` (design DD-2): it needs backlog-domain types and
  replaces the existing `sort_by_key` line in `list_rows`.
- `tests/e2e_backlog_order_golden.rs` (373 lines) — migrate onto `list`; add
  goldens for default-on, opt-out, filtered-compose, and cycle-degrade.

## Risks / Assumptions / Open Questions

- **OQ-1 (RESOLVED, design DD-1)** — **compose-then-filter.** Build the graph over
  the full non-terminal corpus → global sequence; `retain`'s set is ordered by
  position. Forced by the locked decomposition (membership = filter, ordering =
  pure comparator); filter-first would dangle edges and perturb survivor order.
- **OQ-2 (RESOLVED, design DD-4)** — `list --json` emits the **composed sequence**
  (array order = order); the `overrides`/cycle diagnostic stays **out of the JSON
  envelope** (no `json_envelope` change) and routes to stderr as an advisory.
- **OQ-3 (RESOLVED, design DD-3)** — cycle-degrade: id-sort fallback + warning on
  stderr + **exit 0**. The survey verb stays total; never an empty table.
- **Flag spelling (RESOLVED, design DD-5)** — `--by id` (values `sequence` | `id`,
  default `sequence`), not `--no-order`.
- **A-1** — the cordage `BacklogOrder` adapter is reused as-is; the graph-build
  cost on every `list` is acceptable at backlog scale (note, don't optimise).
- **A-2** — today's `list` membership = the `retain` filter (status/kind/tag/
  substr). Ordering must preserve exactly that set; `order`'s non-terminal
  projection is *not* re-imposed as a membership filter.
- **R-1** — surface break: any tooling / docs / skills invoking `backlog order`
  break. Grep + update. Acceptable — internal tooling, clean cut.

## Verification / Closure Intent

- Default `list` emits composed order (golden); the opt-out flag restores id-sort
  (golden).
- A `needs` cycle degrades the default `list` to id-sort + warning — never an
  empty table; total + stderr warn + exit 0 (DD-3).
- Filtered compose: `list --status open --kind improvement` orders the retained
  subset compose-then-filter (DD-1, golden).
- The footer prints only when overrides / drops are non-empty; absent on a clean
  survey.
- `backlog order` is removed — the CLI rejects it; the old golden is migrated, not
  duplicated.
- IMP-028 resolved at close with a resolution referencing SL-051.
- `just check` green; `cargo clippy` zero warnings.

## Follow-Ups

- RSK-005 (`backlog_order` adapter bimap corruption) stays open — flag if the fold
  makes it cheaper to address.
- Cross-kind `needs`/`after` sequencing (IMP-033) is unaffected by this merge.
