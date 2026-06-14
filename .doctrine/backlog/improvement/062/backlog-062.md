# IMP-062: Destructive verbs (delete/archive) for committed authored entities — IMP-006 axis (b)

SL-062 close-time follow-up F1 (design §9). SL-062 delivered the uniform
lifecycle-transition + supersession axis of IMP-006 and CARVED OUT axis (b):
file-level destruction semantics for committed authored entities — archive-status
vs `git rm` vs tombstone (the R2 of the SL-062 scope). Shares the `src/entity.rs`
claim seam. Needs its own design pass.
