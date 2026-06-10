# IMP-020: cordage query.rs traversal triplication: reachable/spine_path/extend_chains diverged walks

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found in the SL-036 post-close code review (codex GPT-5.5 + Opus, independent
agreement). Medium drift smell — no hard divergence bug today, but the shape
that ships one.

**Three near-parallel walks over the same incoming/adjacency structure, with
diverged visited-guard lifecycles:**
- `reachable` (`query.rs:20`) — BFS, `visited` seeded with `start`, never
  cleared → strict, cycle-safe.
- `spine_path` (`query.rs:50`) — linear single-parent walk, `visited` seeded
  with `node`, break on re-entry.
- `extend_chains` (`query.rs:138/173`) — DFS with **backtracking** `visited`
  (insert `:164`, *remove* `:183`) — path-visited, not global-visited — plus
  three chain-termination branches (root, SCC-entry, residual-cycle) vs the
  others' one. The visited-remove is correct for path enumeration but is exactly
  the mechanism behind [[RSK-002]]'s exponential blowup.

Each also re-implements neighbour lookup: `neighbours` / `single_parent` /
`predecessors` (`query.rs:189–230`) are three near-identical in-edge readers.

**Risk:** a future change to one walk won't propagate to the others; the
divergence is how a subtle traversal bug gets shipped (the foreign-node drift in
[[ISS-003]] lives in exactly this seam).

**Improvement:** factor a single typed traversal primitive — shared
"node-exists / predecessor-walk / cycle-guard" — parameterised by the
visited-lifecycle and termination policy. Centralise the in-edge reader.
