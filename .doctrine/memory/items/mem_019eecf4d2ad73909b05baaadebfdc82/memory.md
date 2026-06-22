# Shipped memory authoring flow

Shipped (global-orientation) memories form the doctrine corpus that agents
retrieve via `/retrieve-memory` in any repo. They ship *with* doctrine —
RustEmbed-compiled into the binary and materialized into
`.doctrine/memory/shipped/` by `doctrine memory sync`.

## Authoring flow (project-side, not client-facing)

The CLI/MCP memory tools (`record`, `edit`, `tag`, `status`) operate on
`.doctrine/memory/items/` — the **local corpus**. They cannot write to
shipped memories: `edit` on a shipped uid correctly refuses with "shipped/
is read-only".

Shipped memories are a **separate namespace** with a separate write path:

1. **Edit** `memory/<key>/memory.md` (and `memory.toml` if needed).
   The RustEmbed source is `src/corpus.rs` L44 (`#[folder = "memory/"]`).
   This is the ONLY way to author shipped memories — there is no CLI or MCP
   verb that writes to this directory.

2. **Force re-embed**: RustEmbed has NO `rerun-if-changed` for the `memory/`
   folder. A plain `cargo build` is a no-op unless the embedding crate
   recompiles:
   ```bash
   touch src/corpus.rs && cargo build
   ```

3. **Sync into the repo**: `doctrine memory sync` materializes the embedded
   corpus into `.doctrine/memory/shipped/`. `doctrine claude install` also
   calls this (analogous to how `claude install` refreshes skills).

4. **Clients get it** on their next `doctrine claude install` (or on a
   `SessionStart` hook if `memory sync install` was run).

## Do NOT edit `.doctrine/memory/shipped/` directly

Edits to `shipped/` are **ephemeral** — overwritten on next `memory sync`
or `claude install`. The shipped/ directory is gitignored; only the source
in `memory/` is authored and committed.

## Tool behaviour with shipped memories

| Verb | Works? | Detail |
|---|---|---|
| `find` / `retrieve` / `list` | ✓ | `collect_all` unions items/ + shipped/ |
| `show` (CLI + MCP) | ✗ | `resolve_show` only looks in items/ — bug, see IMP-148 Gap 8 |
| `verify` | ✗ | Same resolution gap |
| `edit` / `tag` / `status` | ✗ | Correctly refused: "shipped/ is read-only" |

## Related

- Skills follow the same pattern: `plugins/` → `touch src/skills.rs` → `cargo build` → `doctrine claude install`
  (`mem.pattern.build.rust-embed-no-rerun`)
- The memory model concept: `mem.concept.doctrine.memory-model`
- IMP-148 tracks the resolve_show fallback bug and MCP tool help gaps
