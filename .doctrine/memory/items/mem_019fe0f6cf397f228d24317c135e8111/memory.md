# A gate probe that deletes a line must restore from a copy, not from git

The shape: "delete row X, run the gate, confirm it reds, restore" — the only
sound way to prove an authored classification row actually bites (SL-248
PHASE-03 T12; the criterion it verifies, `EN-3`, overstates the gate and cannot
be trusted by reading).

The trap: restoring with `git checkout -- <path>` or `git restore -- <path>`
resets the file to **HEAD**, not to the state the probe started from. If the
row under test is itself uncommitted — which it always is, because the probe is
verifying an edit you just made — the restore silently destroys the whole edit.
Every later iteration then measures a file missing *all* the rows and still
reports red, so the loop looks like it is working. Measured on SL-248 PHASE-03:
three iterations, only the first carried any signal; the other two reported six
`Unclassified` violations for a file that had lost every new row.

Two safe forms:

```bash
cp <path> "$SCRATCH/backup"     # snapshot the WORKING TREE
# ... delete, test ...
cp "$SCRATCH/backup" <path>     # restore from the snapshot
```

or commit the edit first, and then `git checkout` means what you wanted.

Generalises past layering: any "break it and watch it fail" experiment on a
file you hold uncommitted edits in. `git` restores from a commit; your baseline
is the working tree.

AGENTS.md already bans `git checkout <ref> --` for a different reason (an empty
pathspec after `--` switches the whole worktree). This is a second, quieter way
the same verb eats work.
