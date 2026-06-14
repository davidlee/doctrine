# Filter a dispatch review branch per-file, not by cherry-pick

Companion to `mem.pattern.dispatch.review-branch-extraneous-deletions`. That memory
says "integrate only the additions, never merge whole" but suggests cherry-pick —
**cherry-picking the phase commits still carries the deletions** (they're part of
each commit's tree). The clean mechanic is per-file, decided by contamination:

1. **Diff `main..review/<branch>` per source file** and split each file's hunks:
   `git diff main review/067 -- <file> | grep -E '^-[^-]'` lists the deletions.
   A deletion that removes legitimate main code the worktree base predates (e.g.
   the REV kind, `is_work_like`'s SL-066 REV widening) is a REVERSION → reject.

2. **Clean file** (only SL-additions; its sole deletions are SL-067's own
   replacements): `git checkout review/<branch> -- <file>` = `main + additions`,
   take it whole. (SL-067: `src/backlog.rs`, `src/listing.rs`, both goldens.)

3. **Contaminated file** (the reverted work AND the new additions both touch it):
   NEVER checkout whole — hand-apply only the additive hunks with Edit. (SL-067:
   `src/main.rs` — applied only the 3 Tag hunks, never the 190-line revision-command
   deletion.)

Footgun met during SL-067 close: `git checkout <branch> -- <file>` **stages** the
file, so a later bare `git commit -m` (no pathspec) sweeps every pre-staged file
into one commit. Commit each group with an explicit pathspec — `git commit <file>
-m …` — or the per-file commit plan collapses.

Verify the whole tree is green AFTER reassembly (`cargo build` + `clippy` + the
gate); per-file correctness does not prove the reassembled tree compiles.
