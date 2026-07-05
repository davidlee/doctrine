# Dispatch funnel arm-unification boundary: shared pure belt, reset-capability-forked impure I/O

The dispatch funnel (import / conclude / reap) runs on two arms — **main-thread
CLI/pi** and **confined-claude (Mode B, SL-199)**. They diverge along **one** axis
only, and the seams are already clean, so future unification is a leg-swap, not a
rebuild.

## Where the arms are already ONE

- **Pure belt shared.** Scope/refusal routes through the single `classify_import`
  (`src/mcp_server/dispatch.rs`, `import_compose` calls it; `run_import` calls it too).
  One verdict, both arms.
- **Funnel MCP tools are transport-agnostic engine wrappers.** `dispatch_import` /
  `dispatch_conclude_phase` / `dispatch_reap` = parse → `run_*`/compose → serialize.
  Nothing in the tool *body* knows stdio / WorktreeCreate / `agent_id`. An out-of-jail
  **http** MCP server exports the identical set.
- **Coord resolved server-side by slice-id** (D-B2), not caller-supplied path —
  already the multi-tenant / out-of-jail shape.
- **Belts don't trust the caller.** Integrity rests on the wall (confinement) +
  server-side belts, not on who called — so a non-stdio transport needs **no new
  authorization model**.

## Where they diverge (the ONE axis) — and why it's unavoidable *here*

Only the **impure apply/commit leg** forks, along **reset-capability**:
- main-thread CLI/pi: `run_import` → `git apply --index` into the live coord tree;
  the orchestrator commits next (it *can* `git reset` on fault).
- confined-claude: bespoke `import_compose` → **working-tree-free** `merge_tree` /
  `commit_tree` / `commit_on_behalf`, because the confined actor cannot reach coord
  `.git` and **cannot reset** — an apply-into-index would strand an unrecoverable
  dirty tree. Conclude likewise reuses only `run_record_boundary`'s **pure**
  `BoundaryRow` compute, not its live-file write.

The working-tree-free primitives (`commit_on_behalf`, `merge_tree` git.rs:854,
`commit_tree` git.rs:828, scratch `GIT_INDEX_FILE` git.rs:725/801) **already exist**.

## The framing correction that matters

The arm asymmetry is **NOT pi-arm-inherent** — it's a **stdio-transport-lands-in-jail**
property. An http (or non-MCP extension) MCP server outside the jail could give a
confined pi orchestrator the same privileged funnel. Uninvestigated, but assumed far
cheaper than getting the claude arm working. So the pi arm is not structurally locked
out of confined orchestration.

## Retrofit is cheap — defer convergence freely

Because the belt is shared and the working-tree-free path already exists, converging
all arms onto working-tree-free compose is a swap of one impure leg behind a stable
seam — not a build. **Two cheap-now / annoying-later framing choices** (do at reconcile,
not now):
1. Phrase §7 governance (ADR-011 D6 amendment, ADR-012 REV) **harness-neutrally** —
   "a confined orchestrator on any harness whose MCP transport sits outside the jail",
   not "the claude confined orchestrator". One sentence now saves a governance round.
2. **Pin a transport-agnostic invariant** on the funnel tools (note/VT): no harness
   assumption in the tool body; harness specifics live only in the spawn/fork seam
   (ADR-011) + the create-fork discriminator. Guards against *rot* re-coupling the arms.

The expensive-later trap is baking "claude" into governance or a shared tool body —
avoided with wording, not work.

See [[mem.pattern.dispatch.worktreecreate-replace-base-control]],
[[mem.fact.dispatch.confined-orchestrator-driveloop-realizable]]. Source-verified this
session (SL-199 PHASE-05 inquisition, `.doctrine/slice/199/inquisition-phase05.md`);
attestation pending scaffold revert (dirty tree).
