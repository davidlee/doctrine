# Concurrent design work on the shared main worktree is expected — don't panic at commingled commits

Multiple agents authoring/designing on this repo's `main` worktree **at the same
time is normal and expected** here, not an ADR-006 violation. ADR-006's
orchestrator-sole-writer rule governs *dispatch into worktree forks* for code
execution — it does not forbid sibling design sessions sharing `main`.

**The benign symptom:** a sibling session running `git commit -a` (or `git add
-A`) in the window between your `Edit` and your own `git commit` will **sweep
your still-unstaged files into its commit**, landing them under a *foreign*
message (e.g. your `SL-045` design edit committed under a `design(SL-046): …`
subject). Observed 2026-06-12: `d87d685` carried both slices' work.

**Why it's safe:** the content is committed and byte-intact — verify with
`git log -S "<your marker>" -- <your file>` and a clean `git diff HEAD -- <file>`.
Only the commit *message* misattributes; nothing is lost.

**Deal with it — do NOT rewrite history.** A rebase/amend/reset to fix the
message races the still-active sibling and can clobber its work. Accept the
commingled commit, note it, move on. The misattribution is cosmetic.

**To avoid it entirely** when you must keep commits clean: stage-and-commit in a
single atomic step (`git add <files> && git commit` back-to-back, no Edits
between), or run the isolated work in a worktree fork (`/worktree`). But for
routine concurrent design on `main`, commingling is tolerated by design.

Related: [[mem.concept.dispatch.gitignored-tier-partition]],
[[mem.pattern.dispatch.fork-rung3-base-not-session-head]].
