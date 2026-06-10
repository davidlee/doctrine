# RSK-001: cordage: Against-orientation U re-map untested by any VT (SL-036 R-D)

The D2 `resolved → oriented` `U` edge re-map (for the F17 eviction key and the
`EvictedEdge` provenance) is implemented, but every SL-036 VT fixture uses
`Along` (oriented ≡ authored), so no test forces the authored-orientation re-map
through `Against`. The path exists and is exercised indirectly; correctness rests
on inspection rather than a dedicated witness — a genuine coverage gap, not a
known defect.

Action for the first `Against` consumer (the adapter / policy slice): add a VT
fixture with an `Against`-oriented overlay and assert the eviction-key selection
and `EvictedEdge` provenance under the re-map. Low risk, low effort.

Refs: SL-036 audit.md R-D; notes.md PHASE-03 ("Against orientation is implemented
but untested by a VT — first Against consumer should add coverage").
