`doctrine worktree coordinate --slice N` fails on the CREATE path (branch doesn't
exist yet) with:

```
Plan for slice NNN not found at <dir>/.doctrine/slice/NNN/plan.toml
```

The plan.toml IS committed on trunk (`git ls-tree trunk .doctrine/slice/NNN/plan.toml`
confirms it), but `git worktree add -b dispatch/NNN dir trunk` doesn't check it out.

**Likely cause:** `.gitignore` has `.doctrine/*` then `!.doctrine/slice/` negation.
The `--git-dir` / worktree checkout path may interact badly with the negated
gitignore pattern, or the sparse-checkout / smudge filter on the worktree is
dropping the file.

**Reproduction (SL-085):**
```sh
git branch -D dispatch/085 2>/dev/null
doctrine worktree coordinate --slice 85 --dir /tmp/test
# → Plan for slice 085 not found at /tmp/test/.doctrine/slice/085/plan.toml
```

**Workaround:** manually create the worktree, provision, and regenerate phases:
```sh
git worktree add -b dispatch/NNN dir trunk
doctrine worktree provision dir
doctrine slice phases --path dir NNN
```

Discovered during SL-085 dispatch setup. The RESUME path (branch exists, no live
worktree) works correctly.

Related: IMP-091 / ISS-016 (import corrupt patch).
