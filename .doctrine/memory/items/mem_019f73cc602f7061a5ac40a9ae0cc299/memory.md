# Close-landing worktree on main needs derived assets + runtime phase state copied from primary

Closing a `/dispatch`ed slice, you land the code+authored state on `main` and do
reconcile+close on a **`main` worktree** (primary stays on `edge`, AGENTS.md). A
plain `git worktree add .worktrees/SL-NNN-close main` checks out only tracked
files — it skips two gitignored-but-required trees, and both failures are
non-obvious:

1. **Derived embed assets (`web/map/dist/**`).** `#[derive(RustEmbed)]` on
   `src/map_server/assets.rs` needs `web/map/dist/` to exist at build time. Absent,
   the macro emits `folder '…/web/map/dist/' does not exist`, `Assets::get` is never
   generated, and you get cascading `E0599 no associated function 'get'` errors (the
   icu/idna candidate spam is a red herring). `.worktreeinclude` lists exactly what
   `doctrine worktree fork` would copy — here `web/map/dist/**` and
   `.doctrine/doctrine.just`. Fix: `cp -r <primary>/web/map/dist
   <worktree>/web/map/dist` (the primary tree has it built).

2. **Runtime phase state (`.doctrine/state/slice/NNN/`).** Phase completion is
   gitignored runtime state in the *primary* tree (symlinked from
   `.doctrine/slice/NNN/phases`). A fresh worktree has none, so `slice list` reads
   the rollup as `—` (untracked) and `slice conformance` reports "incomplete —
   partial coverage / recorded row for PHASE-NN which is not a completed phase". The
   close gate needs `N/N complete`. Fix: `cp -r
   <primary>/.doctrine/state/slice/NNN <worktree>/.doctrine/state/slice/NNN`
   (includes `boundaries.toml` + the `phases/` sheets).

Both copies are of disposable/derived trees — safe, and neither gets committed
(gitignored). Do them right after `git worktree add`, before `cargo build` /
`doctrine check gate` / `slice conformance`.

Note also: memory-record and other corpus verbs resolve to the **primary tree's**
`.doctrine/` regardless of the worktree cwd you run them from (like `record-delta`,
SL-189), so a memory recorded during a `main`-worktree close lands uncommitted on
the primary (`edge`) tree, not on `main`.

Related: [[mem.pattern.dispatch.close-split-lineage-reconcile-on-edge]],
[[mem.pattern.dispatch.close-preff-trunk-absorbs-repair]] (the fix-now-stranded
close traps this session also hit). First observed: SL-222 close, 2026-07-18.
