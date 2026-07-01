# RSK-224: cordage scale_cliffs many_small_cycles_evict_in_linear_time exceeds linear-eviction budget on slow hosts (113s)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Surfaced by SL-183 /audit (RV-209 F-4). `doctrine check gate` fails on
`cordage::scale_cliffs::many_small_cycles_evict_in_linear_time` — 20k disjoint
cycles evicted in 113.5s vs the linear-eviction perf budget (was 67.3s on a
faster host). This is a **slow-host timing** failure, not a logic regression:
the correctness sibling on the identical workload
(`many_small_cycles_evict_set_identical_to_global_loop`) passes.

## Detail

Out of SL-183's diff entirely (`git diff <slice-range> -- crates/cordage` is
empty) — owned by the cordage crate. It makes the full-workspace `doctrine check
gate` non-green on slower hosts. Options for the cordage owner: raise/parametrize
the perf budget for host variance, mark it `ignore`d like its superlinear
sibling (`eviction_fixpoint_scales_superlinearly`), or optimize the eviction
fixpoint for the many-small-cycles workload.

Refs: SL-183, RV-209 F-4, crates/cordage/tests/scale_cliffs.rs:218.
