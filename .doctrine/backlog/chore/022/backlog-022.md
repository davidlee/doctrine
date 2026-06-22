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

Add a characterization test: an `AtMostOne` + `Reject` overlay whose kept-parent
chain re-enters a surviving cycle; assert `spine_path` stops at re-entry (chain
ends, no infinite loop, re-entry node excluded). Low priority — the behaviour is
covered transitively and the input may be hard to construct (pass-1 arity may
evict the cycle before it survives; confirm a surviving cyclic AtMostOne case is
even reachable, else close as not-applicable).
