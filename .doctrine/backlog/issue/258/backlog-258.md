# ISS-258: memory validate rescans the entity catalog per relation: 73s on this corpus

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found while sizing SL-232 objective 7's contribution probe. **Not SL-232's** —
orthogonal to all seven objectives, and in Check 1, which that slice does not
touch. Recorded here so the slice's cost analysis has a citable baseline.

## Measured

```
$ time doctrine memory validate
71.74s user  1.11s system  99% cpu  1:13.21 total
```

99% **user** CPU — this is not subprocess-bound. Single-memory validate
(`memory validate <ref>`) is **0.005s**, so the cost is entirely corpus-scale.

## Cause

`memory_health_findings` Check 1 calls `validate_relation_target(root, target)`
once per relation. That function tries a memory-ref resolution first and, on
failure, falls through to:

```rust
let entities = crate::catalog::scan::scan_entities(root, &mut diagnostics, ScanMode::default())?;
entities.iter().any(|item| item.key.canonical() == target)
```

A **full entity-catalog scan of the whole `.doctrine/` tree, per relation**,
uncached. Complexity is O(relations × corpus size).

At HEAD `377022dfa`, 389 tracked memories carry **777** `[[relation]]` rows:

| target form | count | path |
|---|---|---|
| `mem_…` / `mem.…` | 663 | fast — resolves as a memory ref, no scan |
| canonical ids (`ADR-001`, `SL-232`, …) | **114** | **full `scan_entities` each** |

114 × ~0.64s/scan ≈ the observed 73s.

## Fix

Hoist the scan: build the canonical-id set **once** per `memory_health_findings`
call and membership-test per relation. Expected 73s → under a second. The scan is
already pure w.r.t. the loop — nothing in it depends on the relation being
checked — so this is a hoist, not a redesign.

Care needed on one seam only: `scan_entities` takes a `&mut Vec<Diagnostic>`, so
hoisting changes how many times diagnostics accumulate. Today they are discarded
per call.

## Why it matters beyond the wall-clock

`memory validate` is the corpus-health surface, and **SL-232 objective 7 adds
findings to it** (ISS-257's staleness tri-state, F-36's per-entry contribution
probe). A verb that takes 73s is one agents avoid running, which blunts every
finding it emits. Fixing this raises the value of that objective without being
part of it.

It also reframes objective 7's own cost: the contribution probe adds roughly 440
`ls-files` invocations, which against a 73s baseline is noise — but against the
sub-second baseline this fix produces, it would be the dominant term and worth
measuring again then.
