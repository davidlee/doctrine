# IDE-026: Auto-wire per-phase commit boundary via harness hooks (SubagentStart/bracketing; bwrap/seatbelt wrap)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Follow-up from SL-189 (single-commit boundary primitive).

Once `single_commit_boundary` / `record-delta --commit <S>` exists, the record
step could be automated instead of an orchestrator prose beat:

- **claude arm**: a `SubagentStart` + bracketing (`SubagentStop`) hook could
  capture the worker's diff and wire it straight into a commit boundary
  (`--commit <S>`) without an explicit funnel step.
- **pi/codex arm**: potentially wrap the `bwrap` (Linux) / seatbelt (macOS)
  spawn so the boundary is recorded around the confined worker invocation.

Removes the "remember to record" beat entirely; the boundary becomes a property
of the spawn, not orchestrator discipline. Depends on SL-189 landing the
primitive first.
