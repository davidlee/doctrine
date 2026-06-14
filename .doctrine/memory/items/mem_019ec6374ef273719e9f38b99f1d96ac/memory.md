The Claude `SessionStart` hook merge core in `src/boot.rs`
(`HookSpec`/`plan_hook`/`find_owned`/`hook_array_mut`) is owner-locked and shared
by the boot, memory-sync, and SubagentStart-stamp installs. To add an UNRELATED
top-level key to `.claude/settings.local.json` (SL-064 §8: `worktree.baseRef="head"`),
do NOT thread it through `plan_hook` — write a SEPARATE pure planner + shell
(`plan_baseref`/`install_baseref`) mirroring `plan_hook`/`install_claude_hook`, and
call it BESIDE `install_claude_hook` in `install_refresh`'s Claude arm.

Consequences:
- Two ordered atomic writes to the same file in one refresh: the baseRef step must
  read AFTER the hook write so its plan merges onto current content.
- Mutate `serde_json::Value` at the narrow path (`worktree.baseRef`) — never a typed
  round-trip (drops unknown keys). Preserve `hooks` and sibling `worktree` keys.
- Malformed file / wrong-typed key ⇒ leave untouched (no clobber); a deliberate
  non-`head` value ⇒ report, never overwrite.
- `install_refresh` now returns `RefreshReport { hook, baseref }`; `wire()` reports
  both legs. Codex carries `None`/`NotApplicable` (no worktree baseRef concern).

See [[mem.pattern.distribution.hookspec-merge-core-generalized-event-matcher]].
