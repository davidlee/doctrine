# ISS-484: review new bypasses the review-locus guard the other verbs enforce

`run_new` (`src/review.rs:1341`) calls `crate::root::find` directly and never
`resolve_review_root` (`src/review.rs:2358`), which every other mutating verb
calls. So `review new` mints an id plus an authored entity in a tree where
`prime`, `status`, `raise`, … then refuse with *"review verbs are not supported
on a worktree fork (IMP-024)"*.

Observed: `review new` succeeded in a capsule worktree and produced `RV-369`;
every subsequent verb then refused (obs `01a0adfa`).

Fix: route `run_new` through the same locus predicate (or apply the predicate in
one wrapper over the whole verb family) so a fork is refused *before* allocation.
The locus rule is one decision with 11+ call sites. See RFC-032 `research.md` F5.
