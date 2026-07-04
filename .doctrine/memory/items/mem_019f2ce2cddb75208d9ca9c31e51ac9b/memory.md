# resolve_inspect_uid is items-only; use collect_all + resolve_memory_from_all to resolve a memory key across items+shipped

Resolving a memory reference (key / uid / prefix) to its `mem_<hex>` uid has two
seams with **different tier coverage**. Choosing the wrong one silently fails for
*shipped* memories (the global-orientation corpus in `memory/`, e.g.
`mem.signpost.doctrine.overview`), which are NOT symlinked into `items/`.

- **`memory::resolve_inspect_uid(root, ref)`** (`src/memory.rs`) — calls
  `resolve_show(&items_root, ..)` with **no shipped fallback**. Resolves only
  memories present in `.doctrine/memory/items/`. A shipped-only key returns
  `memory not found`.
- **`memory::collect_all(root)` + `memory::resolve_memory_from_all(&all, &mref)`**
  (`src/memory.rs`) — `collect_all` unions items/ AND shipped/ (items win on
  collision); resolve returns the `&Memory`, take `.uid`. This is the path
  `run_resolve_links` uses, and the correct one for any command that must resolve
  an arbitrary corpus key.

Why `memory show` "works" while `resolve_inspect_uid` doesn't: `run_show` wraps
`resolve_show(items)` in an explicit `.or_else(..)` shipped fallback
(`resolve_shipped_by_key` for a Key, `resolve_show(shipped)` for a Uid) — see
[[mem_019ef1bdda137ea0a992e9d773c6476f]]. `resolve_inspect_uid` has no such
fallback, so verifying resolution via `memory show` does NOT prove
`resolve_inspect_uid` resolves the same ref.

Surfaced on SL-201 (`doctrine onboard` / `map serve --focus mem.<key>`): the
design named `resolve_inspect_uid`; the shipped onboarding key failed at runtime
until switched to the `collect_all` union seam.
