# ISS-030: dispatch sync --integrate leaves stale worktree; close step-3a verify reads ref not tree

Discovered during SL-103 close (2026-06-19).

## Symptom

`/close` step 3a runs `dispatch sync --slice <N> --integrate --trunk
refs/heads/main` from the trunk worktree while `main` is the checked-out branch.
The verb advances the `main` **ref** (ff-only CAS) to the integration merge
commit, but does **not** sync the live index/worktree. HEAD now points at the
new commit (carries the code); the index + worktree stay at the pre-integration
state. Git then reports a phantom **reverse-diff** — the just-integrated source
appears as staged deletions that would *remove* the slice's code.

A naive operator who trusts `git status` here, or runs `just check` without
re-syncing, validates the **stale** tree, not the integrated one. At SL-103 the
post-integrate `just check` returned in 0.06s (cached, old tree) and proved
nothing.

## Two latent gaps

1. **Verb / docs:** `dispatch sync --integrate` is a ref-plumbing verb
   documented to run "from parent/root after the coordination worktree is
   removed." It gives no signal when invoked while the trunk branch is checked
   out, and leaves the worktree desynced from the ref it just moved. No warning,
   no auto-sync, no guidance to re-sync.

2. **Close skill verify is ref-only:** step 3a verifies integration with
   `git diff --stat refs/heads/main~1..refs/heads/main -- src/`. That reads the
   **ref**, so it passes even when the worktree is stale — it cannot catch the
   desync. The skill never tells you to sync the worktree
   (`git restore --source=HEAD -- src/`) after `--integrate`, nor warns about
   running `--integrate` from the trunk branch.

## Manual recovery used (SL-103)

```
git restore --source=HEAD --staged --worktree -- src/   # bring tree up to HEAD
just check                                               # NOW validates real tree
```

Deliberately **not** `git reset --hard` — that would have nuked the unstaged
slice-NNN.toml done-flip and unrelated in-flight work.

## Candidate fixes (pick at design)

- `dispatch sync --integrate` detects trunk-is-checked-out and either refuses,
  warns, or syncs the worktree to the moved ref.
- Close skill step 3a: add an explicit worktree-sync step after `--integrate`,
  and switch the verify to read the **worktree** (e.g. grep the integrated
  symbol in `src/`) rather than only the ref, so a stale tree is caught.

## Relation to ISS-029

Sibling footgun, distinct root. ISS-029 = dispatch **worker** forks the wrong
base at spawn (Bash-cwd-HEAD vs coordination base B). ISS-030 = **trunk ref**
advances under a checked-out branch at integrate, leaving worktree stale. Same
family ("git ref vs working-tree placement" in the dispatch lifecycle), but
different stage and mechanism; ISS-029's cd-into-coord-tree fix does not address
this.
