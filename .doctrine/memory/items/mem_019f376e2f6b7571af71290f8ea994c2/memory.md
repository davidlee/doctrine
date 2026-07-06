# worker_commit's check-commit gate reds in a fork from stale reaped-fork CARGO_MANIFEST_DIR binaries

`mcp__doctrine__worker_commit` runs a full `doctrine check commit` belt
(`just check` = fmt + lint + lint-js + validate + **test** + build) server-side
before landing. In a worker fork that gate can `Refused reason=commit-gate-red`
from **stale test binaries baked with a reaped fork's `CARGO_MANIFEST_DIR`** — a
test compiled against a now-deleted fork's absolute path fails when re-run from a
different fork. Doctrine's own harness **self-flags** it:

```
warning: test binaries baked with the reaped fork's CARGO_MANIFEST_DIR are now
stale — recompile before trusting a RED
error: test failed, to rerun pass `--bin doctrine`
error: recipe `test` failed on line 55 with exit code 101
```

The RED is a **rig artifact**, not a real regression and not the worker's source
delta. Corroboration: at the same base (`ff8c61db`) a freshly-built coord tree's
`check commit` was GREEN (80 suites, SL-206 salvage this session). The `env!`/
`CARGO_MANIFEST_DIR`-baked-path class is the same one CHR-014 / SL-162 ban in
shipped code and SL-206 defers to PHASE-13.

## Consequence

A `commit-gate-red` from a worker fork is NOT automatically a real failure — check
whether the base builds+tests green in a fresh tree before treating it as a
regression. For a clean mode-B commit witness, drive against a base whose test
binaries are freshly compiled in-fork (or a slice whose scratch tree carries no
stale baked binaries) — the natural state of SL-206 PHASE-16's green e2e rig.

Surfaced: SL-206 PHASE-08 P5 probe (`wf_9829cbac-a41`).
Relates: [[mem_019f376ddf9e7a618bbe453c73c66989]] (the P5 mode-B result this
artifact blocked the clean-tip witness of).
