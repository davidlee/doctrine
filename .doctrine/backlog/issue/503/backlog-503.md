# ISS-503: priority --page offset overflows; parallel page helper

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced by RV-412 (SL-275 code review), out of that review's scope.

`priority::resolve_page_offset` (`src/priority/mod.rs:49-63`, used by `survey`
and `next`) computes `(p - 1) * limit` unchecked — a large `--page` panics in
debug and wraps in release. SL-275 added `memory::page_offset`
(`src/memory.rs`), which does the same arithmetic with `checked_mul`.

The two are parallel implementations of one concept with different contracts:
priority lets `--limit 0` (unbounded) coexist with `--offset` and refuses only
`--page` with it; memory refuses `--limit 0` outright. A shared pure helper for
the arithmetic (page → offset, checked) with each surface keeping its own
limit-validation is the likely shape. Also check whether both belong in
`listing.rs` beside `format_truncation_notice`, the shared pagination footer.
