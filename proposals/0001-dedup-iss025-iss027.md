---
seq: 0001
scope: backlog
target: ISS-025, ISS-027
confidence: high
reversible: yes (no entity transition performed; merge/dispose is yours to run)
---
## What
ISS-025 ("add_edge_to_dsl duplicate check is label-based **but parse_dsl dedups by
key**") and ISS-027 ("add_edge_to_dsl duplicate check is label-based, **not
key-based**") are the same defect, captured twice. Both `issue`, both `open`, both
created 2026-06-19, both empty body (`doctrine backlog show` confirms no prose tier
on either). They name the same function and the same mismatch.

The underlying defect is real and singular:
- `add_edge_to_dsl` dup-check matches on raw labels — `src/concept_map.rs:1233-1239`:
  `find(|e| e.from_label == source && e.rel == rel && e.to_label == target)`.
- `parse_dsl` dedups (and validates `DuplicateEdge`) on the **normalised key triple**
  `(from_key, rel, to_key)` — `src/concept_map.rs:421-432`, keys via
  `derive_node_key` (`:268`).
- Consequence: two labels that normalise to the same key (case/whitespace variants)
  pass `add_edge_to_dsl`'s label check, then `parse_dsl` reports a `DuplicateEdge` —
  the very inconsistency both issues describe.

One defect, one fix site. Carrying two cards splits future work and muddies
`backlog list`.

## Options
1. **Merge: keep ISS-025, dispose ISS-027 as duplicate.** ISS-025 has the
   lower id and the richer title (names *both* sides of the mismatch:
   "parse_dsl dedups by key"). Tradeoff: ISS-027's "not key-based" phrasing is the
   crisper fix directive — fold it into ISS-025's body so nothing is lost.
2. **Keep ISS-027, dispose ISS-025.** ISS-027's title is the tighter problem
   statement. Tradeoff: discards the lower, earlier id; mild churn for no gain.
3. **Keep both.** Tradeoff: none positive — guarantees divergent state and a
   double-spend when someone fixes `concept_map.rs:1236`.

## Recommendation
Option 1. Keep **ISS-025** (lower id, title already captures both sides), enrich its
body with the confirmed evidence above (the exact `concept_map.rs` line pair and the
case/whitespace repro), then dispose **ISS-027** as a duplicate of ISS-025.

Decision deferred to YOU: whether to merge at all, and which id survives. I performed
no transition — both remain `open`.

## Next doctrine move
```
# enrich the survivor (optional, recommended):
doctrine backlog show ISS-025          # confirm current state first
#   add body: cite concept_map.rs:1236 (label match) vs :421-432 (key triple),
#   repro = two labels normalising to one key via derive_node_key.

# dispose the duplicate (exact verb/flag: check `doctrine backlog --help`):
doctrine backlog <close|reject|dispose> ISS-027   # reason: duplicate of ISS-025
```
(Verb described, NOT executed — the fence forbids me transitioning a backlog item.)
