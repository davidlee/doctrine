# IMP-091: worktree import produces corrupt patch on git apply --3way --index

`doctrine worktree import` fails with `git apply --3way --index: error: corrupt patch at <stdin>:7`.

Reproduced on a clean `git init` repo (no rtk, no special config):

```sh
git init && git commit --allow-empty -m base
git checkout -b fork && echo x > f && git commit -am delta
git checkout master
doctrine worktree import --base HEAD --fork fork -p .
# → corrupt patch at <stdin>:7
```

The `git diff` output is valid but `git apply --3way --index` rejects it.
Likely cause: how the diff is constructed/piped in `run_import` (worktree.rs ~line 770).

Discovered during SL-085 dispatch. Workaround: `git checkout $FORK -- $CHANGED`
then commit manually.

Related: worker forks imported via workaround cannot be GC'd (landing oracle
uses patch-id matching).
