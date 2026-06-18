# ISS-022: dispatch sync --integrate left staging area in stale reverse-diff state after advancing trunk

During SL-097 close, `doctrine dispatch sync --slice 97 --integrate --trunk refs/heads/main`
advanced main to the admitted `close_target` OID (`fd317597`), but the git staging
area was left in an inconsistent state: SL-097 implementation files (templates,
`src/supersede.rs`, `src/main.rs`, etc.) appeared as staged deletions or
modifications in the reverse direction. A subsequent `git commit` without `git add`
would commit these stale reverts (observed: `0a77cf22` deleted 1038 lines of
SL-097 code alongside an unrelated plan.toml edit).

The trunk was correctly advanced — `git log` showed the right tip — but the index
was not refreshed to match. `git reset --hard HEAD` resolved it.

Suspected cause: the `--integrate` path fast-forwards the trunk ref but does not
check out the working tree or reconcile the index. The pre-existing index state
(before integrate) is left as-is, which may carry reverse-diff staged entries
from prior operations.

May be related to: `ISS-016` (corrupt patch for `git apply --3way`),
`ISS-015` (import verb corrupt-patch), and the general pattern of git index
inconsistencies after dispatch operations.

## Reproduction
1. Have a candidate workflow with admitted `close_target`
2. Run `dispatch sync --integrate --trunk refs/heads/main`
3. Observe `git status` — index may show staged reversions of the integrated changes

## Observed on
SL-097, main at `fd317597` → `git status` showed `M  src/supersede.rs` etc. as staged deletions.
