`receive.denyCurrentBranch = updateInstead` is what makes "provision a repo by
pushing into it" work — the receiving repo checks the pushed commit out into its
worktree instead of refusing. The trap is that the guard governs **only** pushes
to the branch `HEAD` points at.

Push a ref that is not the current branch and none of it applies. The push
**succeeds**, the ref is created, and the worktree is left **empty** — no error,
no warning, and `git log` on the remote looks entirely healthy.

So the receiving repo's `HEAD` is load-bearing transport machinery. If it is
inherited (from `init.defaultBranch`, in config or `/etc/gitconfig`) rather than
set, then:

* anything that changes it — a plain `git checkout -b feature` inside the
  receiving repo — breaks the next provision silently;
* the coupling is invisible in the provisioning code, which looks correct.

**Do this:** point `HEAD` at the ref you are about to push, from the pushing
side, every time — `git symbolic-ref HEAD refs/heads/<ref>` before the push, or
`git init --initial-branch=<ref>` at seed time when the ref is known then. Do not
rely on the receiving repo's default branch matching.

**Verifying it:** a successful push proves nothing here. Check the worktree —
`ls` the target, or `git -C <target> status` — because ref creation and worktree
population are exactly the two things that come apart.

Surfaced in the `RFC-025` capsule round (`EVD-016`), where the guest's
`/etc/gitconfig` set `init.defaultBranch = edge` and made provisioning work for
a reason nothing had written down.
