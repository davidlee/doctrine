# IMP-291: Audit worktree of a review surface lacks gitignored embed assets

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

The `/audit` skill says to prepare the subject in a worktree ("do NOT change the
main repository branch; use a worktree instead"), and for a dispatched slice to
review the `review/<N>` bundle. But a plain `git worktree add <dir> review/<N>`
produces a tree that **cannot `cargo build`**: the RustEmbed `#[folder]` roots
(`web/map/dist/`, and potentially `plugins/`/`install/`/`memory/` when absent)
are gitignored build/derived assets not present in a fresh checkout, so the
`Embed` derive fails to generate `Assets::get` → `E0599`, in files the slice
never touches (`src/map_server/assets.rs`).

Observed auditing SL-211 (RV-274): the review-surface build failed on
`Assets::get`; resolved by manually `cp -r`-ing `web/map/dist/` from the primary
tree, after which the SL-211 suite + clippy ran clean. A silent, confusing trap —
the error points at map-server code with no connection to the slice under audit.

## Fix options

- Have `/audit` provision the worktree via `doctrine worktree provision` /
  `.worktreeinclude` (the copy path that already excludes the coordination tier
  but carries build assets), rather than a raw `git worktree add`.
- Or document the embed-asset copy step in the `/audit` skill's "prepare subject"
  instruction.
- Or make the RustEmbed roots tolerate an absent folder in debug builds.

Adjacent but distinct: IMP-190 (audit worktree-fork review-verb refusal),
ISS-019 (committed plan.toml missing from a new worktree). This one is the
gitignored **embed asset** build failure, not a committed-file absence.
