# Hand-created worktrees skip fork provisioning

A worktree made with raw `git worktree add` (not `doctrine worktree fork`)
skips the provisioning the fork verb does for you. Concrete bite:

- `.worktreeinclude` is bypassed → `web/map/dist/` is absent → `cargo build`
  fails (RustEmbed can't find the embedded map assets).

**Fixes (either):**
- Preferred: create isolation with `doctrine worktree fork` — it provisions.
- If you must hand-roll: copy the assets from a provisioned tree —
  `cp -r <provisioned>/web/map/dist <hand-made>/web/map/dist` (gitignored, no
  commit impact). Then `cargo build`.

Same root cause as the dispatch-worktree family (gitignored build artifacts
absent in a fresh worktree), just triggered by hand creation rather than a
dispatch fork. See the jail-specific gitdir footgun that also bites hand-made
worktrees: [[mem.pattern.platform.jail-host-worktree-gitdir]].
