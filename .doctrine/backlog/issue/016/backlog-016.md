# ISS-016: doctrine worktree import generates a corrupt patch for git apply --3way; checkout-import substitute required

## Symptom

`doctrine worktree import --base <B> --fork <S>` fails:

```
git command failed: apply --3way --index: error: corrupt patch at <stdin>:NNN
```

Reproduced during the SL-068 `/dispatch` run on **both** the PHASE-01 delta
(single file, line ~379) and the PHASE-02 delta (5 files, line ~1031). The
funnel's net-diff `B..S` apply step is unusable.

## Cause (corrected)

Originally attributed to the rtk hook munging git plumbing
([[mem.pattern.dispatch.worktree-import-corrupt-patch-use-checkout]],
[[mem.pattern.dispatch.rtk-git-diff-stat-use-checkout-import]]). **That is
wrong** — the user removed the rtk hook mid-run and `import` *still* corrupts the
patch. The defect is in `import`'s own patch generation / `git apply --3way
--index` invocation (the diff doctrine produces is rejected by `git apply`),
independent of rtk. Likely candidates: missing/whitespace context, binary or
mode handling, or a format `git apply --3way` will not accept (vs `git
diff`/`format-patch` round-trip).

## Workaround in use

The checkout-import substitute lands the delta correctly: replicate the belt
checks manually (single non-merge commit, `S^==B`, no `.doctrine/`/`.claude/`
touch) then `git checkout <fork> -- <changed paths>` to stage the net diff
non-committing onto the coordination index (handle deletes with `git rm`). Used
for SL-068 PHASE-01 and PHASE-02.

## Fix direction

Make `import` generate a patch `git apply --3way --index` accepts, or replace the
apply step with the tree-level checkout-import (stage `B..S` paths from the fork
tree directly — no textual patch). The latter is more robust and matches the
proven workaround. Update the two memories above (cause is doctrine, not rtk).

