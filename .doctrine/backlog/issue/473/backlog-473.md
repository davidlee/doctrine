# ISS-473: knowledge edit list flags sever items containing commas

`doctrine knowledge edit <kind>`'s list-valued facet flags — `--alternatives`,
`--consequences`, `--applies-to` — are documented as *"comma-separated; bare flag
clears"* and are split on a bare `,` with no escape, no repeat form, and no
`@file`/stdin form. An item whose own prose contains a comma is therefore
**silently severed into two array rows**, and the caller's receipt reads clean.

## Observed

`CON-006`'s `applies_to` held three sentence fragments masquerading as rows:

- `" and nothing checks it was honoured at plan time. The plan-side pointer added
  under RV-371 F-4 improves DELIVERY of this obligation; it adds no check"`
- `" so nothing can observe that the precedence was applied or that a named
  second arm was ever raised"`
- `" so a missed capture is unrecoverable and must be reported as unavailable
  rather than estimated"`

Each is the tail of the row above it. `DEC-275`'s `consequences` carries the same
damage. The corruption is invisible at `knowledge inspect`, which renders the
array joined by `", "` — so the round-trip reads back as the text that was
written, and only the raw TOML shows it.

## Why it matters

These are **authored** records: committed, diffable, and cited as the durable
account of a decision. A severed row survives into review as a row that asserts
something no one wrote, and a later `--applies-to` rewrite re-splits it, so the
damage compounds rather than converging. It also makes the flag unusable for a
surgical repair — fixing one id in one row means re-emitting the whole list and
re-inflicting the split, which is why the `CON-006` fix above was a hand-edit.

Related in kind to `STD-003` (no silent skip): a degraded write is disclosed, not
absorbed.

## Candidate directions

- A repeatable flag (`--applies-to` once per item), which removes the delimiter
  entirely and is the shape `--model` already uses elsewhere in the corpus.
- An `@file`/stdin form taking one item per line.
- At minimum, refuse a value whose split would produce an item that does not
  parse as a standalone row, rather than writing it.

## Provenance

Found while repairing `RV-371` `F-3` residue on `SL-260` — `CON-006` cited
`DEC-274` (an unrelated `SL-246` record) where it meant `DEC-275`, and fixing the
id required rejoining the rows the flag had severed.
