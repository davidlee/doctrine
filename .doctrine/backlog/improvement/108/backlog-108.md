# IMP-108: Authored created/updated dates on the spec schema

Specs are the only doctrine kind with **no authored date on disk** — slices, ADRs,
and backlog items all carry `created`/`updated`; specs carry none.

SL-026 (lazyspec read-only projection) surfaced the cost: lazyspec's `DocMeta.date`
is mandatory and must parse `%Y-%m-%d`, so a dateless spec broke INV-7. SL-026 patched
it in the **projection** by injecting the spec toml's filesystem **mtime** as the date
(consult 2026-06-19, the lossy-v1 read-only tradeoff — see SL-026 design §5.3). mtime
is an honest last-changed signal but checkout-unstable across clones; it is provenance
by accident, not by authorship.

The durable fix is **authored dates on the spec model itself** — add `created`
(set at `spec new`) and `updated` (bumped on authored edits) to the spec toml, the
same pattern slices/ADRs/backlog already use. Then the lazyspec loader's existing
`created`-wins precedence makes the mtime fallback dead for specs, and specs gain a
real provenance signal for every other consumer too.

Scope notes:
- Touches the spec scaffold (`spec new`), the `Spec` reader, and any date-bumping edit
  path — a doctrine model change, out of SL-026's read-only scope.
- Backfill: existing specs have no `created`; decide whether to seed from git first-commit
  date, mtime, or leave blank with a tolerant reader.
- Once landed, SL-026's `spec_date` mtime fallback can be simplified/removed.

Relates to: SL-026.
