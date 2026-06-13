# gc squash-merge is indistinguishable from a never-landed fork

The `worktree gc` two-leg landed oracle (ancestry via `merge-base --is-ancestor`
∪ patch-id via `git cherry <coord-HEAD> <fork>`) **cannot detect a squash-merge as
a distinct case.** A multi-commit `git merge --squash` produces a single squash
commit whose patch-id matches NONE of the fork's individual commits, so
`git cherry` lists every fork commit `+` and the fork tip is not an ancestor —
**byte-for-byte the same signal as a fork that never landed at all.** (A
*single*-commit squash is correctly certified landed: its patch-id == the squash
commit's, so `git cherry` reports `-`.)

**Why it matters:** SL-056 design §8.1 reads as if a squash-merged fork should get
its OWN named refusal token. That is structurally impossible — there is no
`cherry`-empty or any other durable git signal that says "squashed". Do not build a
squash detector; it cannot exist.

**How to apply:** collapse squash + never-landed into ONE `not-landed` refusal whose
**message names both remedies** — `--force` / `--superseded-head <SHA>` (the
spent-and-abandoned re-dispatch case) AND "if you squash-merged, re-land via
`worktree land` (--no-ff)". This is faithful to §8.1's actual wording ("trips
neither leg and gc refuses with a **named message**" — a message, never a distinct
token). Verified empirically (`git merge --squash` of a 2-commit fork → `cherry`
all `+`, not-ancestor).

This is the load-bearing reason solo MUST land via the non-squash `land` verb
(§6): squash destroys both gc legs, so a squash-merged fork can never be certified
and gc refuses it. Complements
[[mem.pattern.dispatch.landed-oracle-needs-import-receipt]] (which established
`git cherry` as the oracle method); this records the method's intrinsic blind spot.
