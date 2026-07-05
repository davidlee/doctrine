# IMP-270: dispatch-mechanics.md §Two ways a worker returns mislabels the return arms vs current 3-arm code

Surfaced in SL-199 PHASE-05 (EX-2 Mode B doc work). The §"Two ways a worker
returns" section of `install/dispatch-mechanics.md` encodes a stale **2-arm**
model whose labels no longer match the shipped code's **3-arm** taxonomy:

- **Confined Mode B** — worker self-commits via `worker_commit` MCP →
  `dispatch_import` MCP composes the committed branch.
- **Main-thread claude arm** (the MCP-down fallback) — worker leaves an
  uncommitted worktree → CLI `worktree import --from-worktree`.
- **Subprocess pi arm** — CLI `worktree import --fork`.

`import --help` is the ground truth: `--from-worktree` = "the claude arm";
`--fork` = "the pi/subprocess arm". The doc section currently pairs the labels
the wrong way round (attributes `--from-worktree` to pi).

Left untouched in PHASE-05: broader than that phase's EX-2 (which corrected only
the `dispatch_conclude_phase` two-tier bullet in §"## Mode B"), and it needs the
3-arm taxonomy decision applied consistently across the section, not a spot fix.
Rewrite §"Two ways a worker returns" to the 3-arm model, each arm's return
mechanic cited to `import --help`.
