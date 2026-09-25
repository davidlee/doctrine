# IMP-478: Reconcile ADR-007 D-C10 warm-cache with RFC-004 and SL-147

ADR-007 **D-C10** mandates a reviewer-authored `domain_map` warm-cache keyed on
content-hashes of the explored path set. RFC-004 (resolved) found the prose tier
has zero runtime readers, and SL-147 re-pointed `review prime` at the target
slice's selectors and `review status`→`stale_paths` at the declared-target list.

So tier-2 canon (D-C10) is contradicted by merged implementation, and `IMP-025`
waits for a "second real consumer" of a primitive whose first consumer RFC-004
declared dead.

Decision needed: revise ADR-007 D-C10 to the lived model (selector-derived
path-set, prose tier retired), or re-affirm it and build the reader. Also settle
D-C10's deferred worktree-aware staleness model. See RFC-032 `research.md` F7/A4.
