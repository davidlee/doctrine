# IMP-043: import verb: moved-HEAD re-anchor (--allow-reanchor) — 3way onto moved coordination HEAD + computable path-disjointness

## Problem

The dispatch funnel binds `worktree-base == --base == coord-HEAD` rigidly.
When an authored correction is committed to the coordination branch between
`arm-spawn` and `import`, coord HEAD advances off the worker's fork base and
`verify-worker` reports `wrong-base` — forcing a full re-fork and re-dispatch
even when the intermediate commit touches files **disjoint** from the worker's
delta.

Witnessed in SL-193: a design-note commit and a selector-add commit each
advanced coord HEAD, stranding the in-flight worker twice (~191k tokens in
re-dispatches). See IMP-257 for the prevention side; this item is the
**structural fix**.

## Fix direction

- **`import --allow-reanchor`**: when the coordination HEAD has moved from the
  pinned base B, compute whether the intermediate commits are **path-disjoint**
  from the worker's delta. If they are, 3-way merge the delta onto the new HEAD
  (via `git apply --3way` with the moved base as the common ancestor).
- If they are NOT path-disjoint, refuse with a clear message naming the
  conflicting paths — same fail-closed posture as today's `wrong-base`, but
  with a richer diagnostic.
- Pure path-disjointness is a cheap gate: `git diff B..HEAD --name-only` ∩
  `git diff B..S --name-only` = ∅ → safe to re-anchor.
- Gated behind a flag (`--allow-reanchor`) so the default remains strict;
  orchestrator opts in when it knows the intermediate commit was a
  design/selector correction.

## Related

- IMP-257 (mid-drive authored-commit guard — the prevention/skill side)
- IMP-256 (selector completeness — removes the most common driver)
- ISS-015, ISS-016, ISS-026 (related import pipeline fixes, all closed)
