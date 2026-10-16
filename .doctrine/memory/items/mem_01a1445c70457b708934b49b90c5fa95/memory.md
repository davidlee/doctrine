The project rule is to path-limit every commit (`git commit <paths> -F -`) so
another agent's staged files never ride along. That form uses `--only`
semantics: it commits **each named path as it is in the working tree**, not as
it is in the index.

So untracking a file that still exists on disk does not survive it:

    git rm --cached foo        # index: foo deleted
    git commit foo -F -        # commits foo's worktree content → re-adds it

The commit "succeeds", reports no change for `foo`, and the file stays
tracked. Seen in SL-281 (RV-420 F-5) untracking `.pi/agents/dispatch-worker.md`.

**How to apply:** for an untrack, run `git status --porcelain` and confirm
the index holds *only* your `D ` entry, then commit the index unscoped
(`git commit -F -`). The porcelain check is what the path-limit rule exists to
guarantee; doing it by hand satisfies the rule's intent. Verify afterwards with
`git ls-files <path>` (empty) and `git check-ignore -v <path>`.
