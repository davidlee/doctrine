Verified empirically by headless `claude -p` probe on **v2.1.220** (SL-250
pre-design research round, 2026-08-06). Three facts about hooks declared in
`.claude/settings.json` / `.claude/settings.local.json`:

1. **Same command, different matcher → both fire.** Two `PreToolUse` entries
   whose `command` strings are byte-identical, differing only in `matcher`, are
   read and fired independently, each receiving its own tool in the stdin
   payload. There is no dedup-by-command at the harness layer.

2. **Scopes MERGE; they do not override.** A hook in project
   `.claude/settings.json` and a hook in `.claude/settings.local.json` on the
   *same* matcher both fired on one tool call. This matches the documented
   array-merge rule ("concatenated and deduplicated"), which exempts exactly two
   settings — `fallbackModel` and `availableModels` — neither of them `hooks`.

3. **`${VAR:-default}` expands in a hook command.** With the variable unset the
   default branch ran, so `${DOCTRINE_BIN:-doctrine} <verb>` works verbatim in a
   settings file, as it already does in `.mcp.json`.

## The trap that falls out of (1) + (2)

Doctrine's owner-locked merge core proves ownership by **command alone**
(`src/boot.rs` `owned_positions`) and normalizes to one canonical entry **within
the single file it writes** (`install_hook_to_file(root, rel_path, …)`).

- Against (1): one predicate per command marks every same-command entry owned
  and the normalize drops all but one. Multi-matcher hook sets cannot be
  expressed until ownership widens to `(command, matcher)`.
- Against (2): a stale owned entry in `settings.local.json` plus the same hook
  in `settings.json` **both fire** — a per-file normalize cannot see across the
  pair. Any scope switch must evict the entry from the scope it leaves.

Also relevant: hooks are **hot-reloaded** by the settings file watcher (docs,
*When edits take effect*) — no restart and no `/reload-plugins` equivalent
needed. And `strictPluginOnlyCustomization` (managed settings only) blocks hooks
from user and project sources while plugin hooks survive — the one gate where
the plugin channel is safer than direct-write.

See also [[mem.concept.claude.trust-layers]].
