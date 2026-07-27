# IMP-328: pi spawn fork is unbound to slice and phase

`scripts/pi-spawn-confined.sh` (and the shipped `dispatch-subprocess` skill it
mirrors) forks with:

```sh
"$DOCTRINE" worktree fork --base "$B" --branch "$BR" --dir "$D" --worker
```

No `--slice`, no `--phase`. Per `worktree fork --help`, those two flags plus a
`<coord>/.worktrees/<name>` dir are what create the **durable fork binding**
(SL-228 PHASE-04): "without it the fork is unbound and downstream verbs refuse
`unprovable-fork`".

The caller also chooses `$BR` and `$D` freely. The funnel's import resolver
(`worktree/dispatch_record.rs::resolve_agent`) does not accept a free branch: it
sanitises the agent id, keys `refs/heads/dispatch/<agent>` through
`git::worktree_for_ref`, recovers coord by stripping `.worktrees/<name>` from the
resolved dir, and reads the per-agent record. So a funnel-importable pi fork must
be exactly:

- branch `dispatch/<agent>`
- dir `<coord>/.worktrees/<agent>` (basename == agent)
- forked with `--slice N --phase PHASE-NN --worker`, run from the coord tree

None of that is expressed in the script's interface, which instead documents the
older free-form convention (`w/SL-186-p03`, `w/SL-192-p01` in the live worktree
list are all of that shape and all funnel-unresolvable).

## Observed

SL-231 PHASE-01. Fork created as `w/SL-231-p01` at
`<primary>/.worktrees/SL-231-p01` per the script's documented usage. The worker
ran fine and produced a good delta, but `dispatch_import` refused
`unknown-agent`. Recovery was cheap — re-fork bound under coord, cherry-pick the
single worker commit (byte-identical tree), retry — but it is pure waste, and it
is waste every pi-arm phase pays.

The claude arm does not hit this: it goes through `dispatch arm-spawn` +
`worker_commit`, which own the naming and the binding.

## Fix

Derive the branch and dir inside the script from `(slice, phase)` rather than
taking them as free positional args, and pass `--slice`/`--phase` through to
`worktree fork`. The script already resolves `$ROOT`; it needs the coord dir too
(`dispatch status` prints it, or accept it as an arg). Ideally the positional
`<B> <BRANCH> <DIR>` interface collapses to `<slice> <phase>` with the base read
from the coord tip, so the funnel-legal shape is the only representable one.

Update `dispatch-subprocess/SKILL.md`'s spawn snippet in the same change — it
carries the same unbound `worktree fork` line, so any agent following the skill
reproduces the defect.

Relates to ISS-260 (also surfaced driving SL-231 PHASE-01 on the pi arm).
