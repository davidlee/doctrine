# Default backlog list orders by ItemId prefix-string, diverging from --by id grouping

SL-051 folded ordering into `backlog list` as a default-on comparator (`--by
sequence`, the default; `--by id` is the opt-out). Non-obvious consequence for
tests and goldens:

When the corpus has **no `after`/`needs` edges** (or rows share a created date),
`--by sequence` does **not** fall back to `--by id`'s `(kind.ordinal, id)`
grouping. Every non-terminal row gets a composed position from
`BacklogOrder::ordered()`, whose tie-break for unconstrained nodes is
`(exposure desc, created asc, ItemId asc)` — and **`ItemId` Ord is by prefix
STRING**: `IMP` < `ISS` < `RSK`. So default `list` interleaves kinds by prefix
letter (e.g. `IMP-009` before `ISS-002`), whereas `--by id` groups by
`kind.ordinal` (issue=0 before improvement=1, etc.).

Implications:
- A test asserting a **multi-kind** backlog list order must pick `--by id` if it
  wants the classic ordinal grouping; default/`--by sequence` will look
  "shuffled" by prefix string.
- Only rows the graph never placed (terminal rows surfaced via `--all`/`--status`)
  tail by `usize::MAX` then `(kind.ordinal, id)`.

See [[mem.pattern.cordage.opaque-ids-capture-from-builder]] and
[[mem.pattern.testing.black-box-cli-golden]].
