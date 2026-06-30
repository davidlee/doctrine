# CHR-030: Candidate worktree lacks gitignored embed assets (web/map/dist); bin fails to compile until copied in

Surfaced by the SL-172 audit (RV-189), re-hit during SL-177 audit.

## Root cause

`dispatch candidate create --worktree` (`src/dispatch.rs:1259`,
`add_candidate_worktree`) creates a worktree via `git worktree add` but does
NOT call `run_provision` to copy allowlisted gitignored files from
`.worktreeinclude`.

By contrast, `doctrine worktree fork` (`src/worktree/fork.rs:147`) DOES call
`run_provision`, which reads `.worktreeinclude` and copies allowlisted
files — the `fork` path already works correctly.

## The failing asset

`src/map_server/assets.rs:15` — `#[derive(RustEmbed)] #[folder = "web/map/dist/"]`.
`web/map/dist/` is gitignored (`.gitignore:73`) and listed in `.worktreeinclude`
as `web/map/dist/**`. When absent, RustEmbed emits no `Assets::get` →
`error[E0599]: no associated function 'get' found for struct Assets` → bin won't
compile.

No other embed roots are affected: `install/`, `memory/`, `plugins/` are all
tracked in git.

## Fix

Call `run_provision` after `add_candidate_worktree` in `candidate_create`
(`src/dispatch.rs`). The provision machinery already exists and is proven (used
by `worktree fork`). `.worktreeinclude` already lists `web/map/dist/**`.

De-risked (2026-06-30 preflight):
- `run_provision` is `pub(crate)` via `crate::worktree::run_provision`
- `verify_sibling_worktree` will accept the candidate path — it's a linked
  worktree sharing the same git-common-dir
- `run_provision` → `root::find` with `Some(root.to_path_buf())` finds the
  already-resolved root
- Provision failure should roll back the branch (fail-early: a candidate that
  can't compile is useless)
- `copy_selected` in `fsutil.rs` handles existing files gracefully (overwrites)

## Related

- CHR-020: same topic, empty body, likely stale (fork path fixed by SL-116)
- RFC-011 case notes: lines 285–293, 382–394, 445–451, 584–589
- RV-189 (SL-172 audit discovery)
- Memories: `mem_019eeac33cf373d3949d04a6f9780351`,
  `mem_019eec3285e471c287a0c3d74c235b25`,
  `mem_019f068fd92e7b9184d5615220d25233`
