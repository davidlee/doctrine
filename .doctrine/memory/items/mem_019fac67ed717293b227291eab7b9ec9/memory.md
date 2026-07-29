# The `NNN-slug` symlink doubles any naive recursive walk

`doctrine <kind> new` mints a **title-slug symlink** beside the numbered entity
directory as a convenience (`.doctrine/knowledge/decision/001-my-title` →
`001`), and the symlink is committed with the entity. AGENTS.md documents the
symlink; nothing documents its consequence for readers.

**The consequence.** Any recursive directory walk that does not skip symlinks
visits every entity **twice** — once as `001/record-001.toml` and once as
`001-my-title/record-001.toml`. Both paths exist, both are readable, both parse.

**Why it is worth a memory: the failure reads like a different bug.** SL-233
PHASE-05 had two black-box tests fail on first run with

```
assertion `left == right` failed: one record exists to adopt
  left: 2
 right: 1
["…/decision/001/record-001.toml", "…/decision/001-checkpointed-decision/record-001.toml"]
```

That is exactly what a genuine duplicate-record defect looks like, in a phase
whose whole point was "creates no duplicate". Two of five tests were chased as
implementation defects before the walker was identified as the fault. The cost is
paid by the reader, not the writer.

**How to apply.** Any test or tool that counts, enumerates, or hashes entities on
disk must skip symlinks — check `std::fs::symlink_metadata(...).is_symlink()`
(NOT `metadata`, which follows the link), or filter the walker. Prefer counting
via the CLI's own listing verbs, which already resolve identity correctly, over
walking `.doctrine/` by hand.

Related: [[mem.system.engine.identity-claim-seam]] (the engine's two identity
shapes and the `write_fileset` H1/H2 contract).
