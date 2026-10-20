# IMP-521: Symlink-safe settings writes on the Claude and codex arms

SL-284 PHASE-04 found that `fsutil::write_atomic` followed a symlinked settings
file on read and then renamed over the **link** — severing a dotfiles-managed
`settings.json` while leaving its target stale. The fix landed on the pi arm
only: `boot::install_pi_settings` now checks `symlink_metadata` first and takes a
disclosed manual fallback (`PiSettingsOutcome::ManualFallback`), tested by
`symlinked_pi_settings_falls_back_without_replacing_the_link`.

The identical hazard is still live on the two older arms, because they reach
`write_atomic` through the shared hook/settings merge core:

- Claude: `.claude/settings.json` / `.claude/settings.local.json` (hook merge,
  `worktree.baseRef`) and `.mcp.json`;
- codex: `.codex/hooks.json`, `.codex/config.toml`.

This is pre-existing, not introduced by SL-284 — which is exactly why it should
not be quietly forgotten once the pi arm looks consistent. Either hoist the
symlink guard into the shared settings-merge seam (`parse_settings`'s callers) so
the three arms cannot disagree, or give each arm the same disclosed fallback;
the former is the smaller surface.
