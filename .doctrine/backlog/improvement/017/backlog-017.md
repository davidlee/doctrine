# IMP-017: memory list adopts the shared column model

## Context

SL-037 introduces the shared list **column model** (`Column<R>` + `select_columns`
+ `render_columns` on `src/listing.rs`) and the `--columns` projection flag on the
shared `CommonListArgs`, migrating the four slug-bearing list verbs
(backlog/slice/spec/governance). Memory was deliberately left **out** of that
slice.

Why deferred (SL-037 design §OQ-3 / R4):

- Memory has **no slug** — IMP-009's driver (hide the noisy slug column) does not
  apply, so migrating memory yields zero default-output change.
- Memory's table cells are **security-scrubbed** (`scrub_line` over trust/key/title
  — hostile-input defense, memory-spec § Security). A generic column model inserts
  an abstraction between "column" and "must scrub" — a seam where a future column
  could skip the scrub. Routing the most divergent, security-sensitive verb through
  the new abstraction needs its own justification, not a uniformity reflex.
- It is the strongest case of the over-abstraction IMP-013 warned against (memory
  is keyed not numbered, carries trust/severity axes, shares the retrieve/find
  pipeline).

Consequence carried by SL-037: `--columns` is accepted-but-ignored on `memory
list` (documented no-op), since the flag rides the shared `CommonListArgs`.

## Trigger (deferred-until-condition — see IMP-012)

Fires when a phase **next reshapes `memory list` rendering** (`format_rows` /
`json_rows` in `src/memory.rs`), OR when the accept-but-ignore `--columns` no-op on
memory becomes a felt inconsistency. At that edit, adopting the SL-037 column model
(memory declaring its own column set: `uid type status trust key title`) is cheaper
than re-diverging — provided the `scrub_line` invariant is carried into the column
extractors and pinned by a security test.

Path trigger (pending IMP-012's structural field): `src/memory.rs` — `format_rows`
/ `list_rows`. Until IMP-012 ships, this prose IS the trigger.

## Relations

Descends from the SL-037 column-model surface. Sibling deferred-lift precedent:
IMP-013 (slice/spec lift, triggered by SL-037). Security invariant to preserve:
`scrub_line` cell scrubbing.
