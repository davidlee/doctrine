# New authored entity type needs manifest dir + gitignore negation

When you add a new **authored** entity type under `.doctrine/` (adr, spec,
requirement, …), two surfaces must be wired or the tree is silently broken:

1. **`install/manifest.toml` → `[dirs].create`** — add `.doctrine/<type>` so a
   fresh install scaffolds the authored tree, for parity with `slice` /
   `memory/items`. (Functionally the entity engine mkdir's on demand, so this is
   discoverability/parity, not strictly required for the `new` verb to work.)

2. **`.gitignore` negation** — *this repo* uses a blanket `.doctrine/*` ignore
   with per-tree negations (`!.doctrine/slice/`, …). A new authored type is
   **silently uncommittable** until you add `!.doctrine/<type>/`. `git add`
   fails with "paths are ignored". The installer's denylist model (other
   projects) does NOT have this trap — it only ignores specific runtime/derived
   paths, so authored trees are tracked by default.

Bit `adr` (SL-006 shipped the command, never wired either surface — ADR-001 was
uncommittable until fixed) and was pre-empted for `spec`/`requirement` (SL-015).

Derived/runtime subtrees are the inverse: ignore them narrowly (e.g.
`.doctrine/memory/{index,embeddings,state}/*`), never blanket the parent.
