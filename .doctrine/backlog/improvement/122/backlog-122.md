# IMP-122: Harden integrate None-leg post-CAS resync: re-resolve target before reset --hard, guard untracked collisions

Surfaced by the SL-121 audit (RV-107 F-1, F-2; codex GPT-5.5 external pass). Two
narrow, race-gated hardenings to the integrate **None-leg** post-CAS resync
(`advance_pure_ref` → `resync_worktree_hard`, `src/dispatch.rs:1217` /
`src/git.rs:1236`):

1. **Re-resolve before resync (F-1).** After the post-CAS re-probe finds the ref
   newly checked out, re-resolve `target_ref`; if it is no longer `planned` (a
   concurrent writer advanced it past `planned` in the CAS→resync window), set the
   row `Failed` + return a captured raced refusal and do **not** `reset --hard` —
   else the hard reset forces the live branch back to `planned`, silently clobbering
   the concurrent advance while reporting success.
2. **Untracked-collision guard (F-2).** `reset --hard` overwrites an untracked
   file/dir that collides with a tracked path in the target tree (no "would be
   overwritten" abort, unlike the ff-merge leg). Guard untracked collisions before
   the hard reset and downgrade to a `raced-checkout-desync` warning, or always warn
   rather than hard-reset on the raced leg.

Both close honesty gaps in design §7's "all residual races are content-safe" claim
(corrected at SL-121 reconcile). Vanishing likelihood (the None-leg resync only
fires when a ref becomes checked out *after* the CAS — not the normal
checked-out-trunk close path), single-writer close narrows it further. The broad
elimination (a real worktree/placement lock) is the larger §7 follow-up; this item
is only the cheap local hardening. Related: SL-121.
