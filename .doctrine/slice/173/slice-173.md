# backlog list --after / --needs dependency-sequence edge filter

## Context

`doctrine backlog list` can filter by substr/regex/status/tag/kind, and
optionally order by the composed `needs`/`after` graph (`--by sequence`). But
there is no way to ask: "show me every item that declares an `after` edge
pointing to IMP-194" or "every item that `needs` SL-169". The `after`/`needs`
typed axes are authoring-only via the `backlog after`/`backlog needs`
subcommands; `relation list --label after --target X` only covers tier-1
`[[relation]]` edges (which backlog `after` is not). The query is
inexpressible.

The data is already in memory — each `BacklogItem.relationships` carries
`after: Vec<AfterEdge { to, rank }>` and `needs: Vec<String>`. No new I/O.

## Scope & Objectives

- Add `--after <REF>` (repeatable, OR logic) to `backlog list` — retains only
  items whose `relationships.after[].to` matches any given ref.
- Add `--needs <REF>` (repeatable, OR logic) — retains only items whose
  `relationships.needs` contains any given ref.
- Both filters compose with the existing filter axes (substr/regex/status/tag/
  kind) via AND — as all other filters do.
- The terminal hide-set (`resolved`/`closed` hidden by default) still applies.
  Combine with `--all` or `-s resolved,closed` to see everything.
- Table, JSON, and `--columns` all work as usual — only row membership changes.

## Non-Goals

- No new ordering mode — `--by sequence` is unchanged (it already uses the
  `after`/`needs` graph).
- No `--after` on other entity kinds (slices, ADRs, specs). Backlog only.
- No `relation list` changes — those cover tier-1 edges, not dep/seq.
- No transitive closure ("what transitively comes after X"). Direct edges only.
- The edge refs are matched as raw authored strings — no resolution to live
  items. A ref to a deleted/dangling item still matches.

## Affected surface

- `src/backlog.rs` — CLI enum `BacklogCommand::List`, pipeline `run_list` →
  `list_rows`, filter step after `retain`
- Tests in `src/backlog.rs` — existing `list_rows` tests extended

## Risks & assumptions

- **Risk**: adding `--after`/`--needs` as repeatable `--after A --after B` may
  be less ergonomic than comma-separated `--after A,B`. Start with repeatable
  (consistent with `--tag`), revisit if feedback disagrees.
- **Decision (design D1)**: refs match **normalized** — parse to `(kind, id)`
  via `parse_canonical_ref` and compare, with a verbatim fallback for
  unparseable authored refs. No existence resolution, so a dangling/deleted ref
  still matches; normalization adds no I/O. (Supersedes the earlier raw-string
  assumption — case/padding now absorbed, e.g. `--after imp-0194` matches
  `IMP-194`.)
- **Assumption**: the terminal hide-set should NOT be overridden — consistency
  with other filter axes.

## Verification

- `doctrine backlog list --after IMP-194 --all` on a repo with `after` edges
  pointing at IMP-194 — verify only those items appear.
- Combine `--after` with `--status` — verify AND composition.
- Combine `--after` with `--needs` — verify AND composition (item must have
  BOTH an `after` and a `needs` edge).
- `--by sequence` still produces the correct composed order on the filtered
  set.
- Table, `--json`, and `--columns` all render correctly.
