# CHR-022: cordage spine_path: direct test on cyclic AtMostOne Reject overlay

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced by SL-140 (finding R2). After SL-140 unified `reachable` + `spine_path`
onto `walk_bfs`, `spine_path`'s cycle-stop rides the shared primitive, whose
cycle-safety is *directly* asserted only by `reachable`'s a↔b test
(`reachability.rs` l.82–83). There is no direct assertion of `spine_path` walking
into a surviving `Reject` cycle on an `AtMostOne` overlay — it's covered only
transitively.

## Resolution (2026-06-24): closed / obsolete

**Spike confirmed:** a surviving cycle IS reachable — a simple 3-cycle A→B→C→A
in an `AtMostOne`+`Reject` overlay has exactly 1 incoming edge per node, so pass-1
arity does nothing, and `Reject` diagnoses but doesn't evict.

**Behaviour is correct:** `spine_path(A)` → [B, C, A].  `walk_bfs`'s visited set
stops the walk at A (B's parent, already visited).  The chain terminates cleanly
with no infinite loop.  This invariant is **directly** tested by
`reachable_terminates_and_stays_strict_on_a_reject_cycle` (reachability.rs
L72–85) — a `spine_path`-specific characterization test would assert behaviour
that cannot fail without first breaking the `walk_bfs` primitive test.

Closed as obsolete — transitively covered, no marginal value in adding the test.
