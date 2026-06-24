# IMP-167: memory list --long: card-style renderer as alternative to table

The `doctrine memory list` table is cramped — the `uid` column (44 chars) and
long `key` columns push everything wide. A `--long` flag on `MemoryCommand::List`
would render each memory as a labeled block with a compact metadata mini-row,
avoiding the wide table layout.

## Proposed format

```
---
title: Memory entity v1
key:   mem.fact.git.remote-mutation-seam
uid:   mem_019ef779ed7971d2a88ac19ef2fb601e
type   trust   status
fact   medium  active
---
```

- Colours preserved via existing `memory_type_hue`, `trust_hue`, `status_hue`
  (already `pub(crate)` in `listing.rs`)
- `key` omitted when None
- `--long` overrides `--format`/`--json`; `--columns` ignored (fixed layout)
- All in `src/memory.rs` — no `Format` enum or other list surface touched

## Preflight notes (2026-06-24)

- `--long` flag on `MemoryCommand::List` + new `render_memory_long()` + dispatch
  branch. ~110-150 lines, all in `memory.rs`.
- No `Format` enum change — avoids touching 27 call sites.
- Can be implemented as a standalone slice or bundled as a small chore.
