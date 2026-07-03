# Host-created worktree gitdir breaks in-jail

A worktree created **on the host** (outside the bubblewrap jail) writes an
**absolute host path** into its gitdir pointer:

```
<worktree>/.git   →   gitdir: /home/david/dev/doctrine/.git/worktrees/<name>
```

That path doesn't exist under the jail's `/workspace/...` mount → `git` fails
`not a git repository` in-jail.

**Fix — relativize both directions so it resolves under any prefix:**

- worktree `.git` → `gitdir: ../../.git/worktrees/<name>`
- admin back-ref `.git/worktrees/<name>/gitdir` → `../../../.worktrees/<name>/.git`

Then `git` works from host **and** jail with no further edits.

**Caveat:** if a later *commit* fails with a read-only / permission error (NOT
`not a git repository`), the jail ro-binds the shared `.git` — different problem;
needs a standalone clone, not a path fix.

Worktrees created with `git worktree add` from **inside** the jail don't hit
this (paths resolve correctly from the start). Sibling footgun for hand-made
worktrees: [[mem.pattern.platform.hand-created-worktree-provisioning]].
