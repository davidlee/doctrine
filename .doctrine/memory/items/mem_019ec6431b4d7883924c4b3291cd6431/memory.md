# rtk masks git plumbing — funnel re-anchor proofs must bypass via rtk proxy git, checkout-import unsafe under real overlap

Hit integrating SL-066 onto a moved `main`. The `dispatch sync --integrate` FF-CAS
refused (trunk moved), so the close fell back to the manual funnel re-anchor
([[mem.pattern.dispatch.reanchor-base-on-disjoint-head-move]] /
[[mem.pattern.dispatch.three-way-import-onto-moved-shared-main]]). Under the rtk
hook, the git plumbing those proofs rely on is **silently corrupted**:

- `git diff --name-only -- <many paths>` returned EMPTY for paths that had moved
  (false disjointness — `src/main.rs` had +21 SL-064 lines but read as unchanged).
- `git rev-parse <rev>:<path>` and `git ls-tree` returned phantom hits / misses
  (a new file read as "present on main"; a present file's `git log` read empty).

Acting on the false disjointness, a `git checkout <S> -- <paths>` import (the
rtk-safe shortcut for *disjoint* batches,
[[mem.pattern.dispatch.rtk-git-diff-stat-use-checkout-import]]) **reverted main.rs's
SL-064 wiring**, turning `run_verify_worker`/`classify_worker_verify` into dead code
— caught only by `just check` going red on the combined tree.

## How to integrate safely under rtk

1. **Bypass rtk for every plumbing query**: `rtk proxy git rev-parse|ls-tree|log|
   diff …`. The hook rewrites/stats plain `git`; the proxy returns raw. Pristine-tree
   health, the overlap set, and "is it already landed?" must ALL be read raw.
2. **Prove overlap per-path with raw blob compare**, not `--name-only`:
   `rtk proxy git rev-parse $OLDB:$f` vs `…$HEAD:$f`.
3. **`checkout-import` is ONLY for proven-disjoint paths.** Any path that moved on
   main needs a real 3-way: generate the raw patch (`rtk proxy git diff $OLDB $S >
   p.patch`), `git apply --3way --index p.patch` (this merges the moved `main.rs`
   with the bundle's additions when regions are disjoint — `merge-tree` 0-conflict
   predicts it), commit WITHOUT `-a`.
4. **The combined-tree `just check` is the real gate** — it, not the import
   mechanics, is what caught the reverted wiring. Never trust the import's own
   fidelity check alone.
