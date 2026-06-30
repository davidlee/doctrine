# Memory key→uid resolution ignores status — retiring a capture needs the items/ alias symlink physically removed

`memory show <key>` resolves `<key>`→uid via the `items/mem.<key>` alias symlink
and returns the **local capture even when it is `archived`/`superseded`**. The
status flag filters `memory find` only — it does NOT gate key→uid resolution. So
marking a capture retired does not re-point `<key>`.

To make `<key>` resolve to a shipped master instead, you must physically remove
the `items/mem.<key>` alias symlink (`git rm`); a status change alone is
insufficient. The shipped master under `memory/` then wins key resolution via the
`shipped/` fall-through (see [[mem.fact.doctrine.show-shipped-by-key]]).

**Corollary — supersede cannot name a shipped successor.** `memory status
superseded --by <successor>` resolves `<successor>` against `items/` only
(`src/memory.rs` resolve_uid_prefix; "items/ wins any uid collision"). A shipped
master lives under `memory/`→`shipped/`, never `items/`, so it can never be named
as a supersession successor — the call fails "memory not found". The working
local-capture→shipped-master retire mechanism is therefore: scrub the body
([[mem.pattern.doctrine.shipped-master-body-scrub]]), mint the master via `memory
record --global`, then `git rm` the items/ key-alias symlink + `memory status
archived` the capture uid dir.

Discovered in SL-178 PHASE-01 (the crux that forced a `/consult`); RV-196 F-1/F-2.
